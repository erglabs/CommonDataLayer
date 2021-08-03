# Materialized views
 
## Motivation

CDL lacks the ability to filter and transform data before returning it to the user. The purpose of this proposal is allowing users to define views which would allow them to receive data in different format then it was inserted to CDL. Because creating some complex views on the fly can be computationally expensive, we are introducing concept of materialized views – views which will be precalculated, but may not always contain the latest data. Materialized views will be refreshed automatically on user defined conditions (for now those setting will be defined per deployment). 
***
:warning: Implementing this proposal requires implementation of  [CDLF-00012-00](https://github.com/epiphany-platform/CommonDataLayer/blob/develop/docs/rfc/CDLF-00012-00-rfc-01.md)
***
## Proposed changes
- Add 3 new components: 
    - Partial Update Engine – responsible for deciding if materialized view data needs to be updated, if we need to recalculate whole view or if we can keep it up to date by changing only few entries 
    - Object builder – responsible for fetching data from various repositories and joining it together 
    - Materializers – responsible for storing materialized view data in user specified service (e.g. Elasticsearch, Postgres) 
- Change view definition structure stored in schema registry 

## Feature list
- Automated materialized view updates - Partial Update Engine 
- Limit how often views can be recalculated – Partial Update Engine 
- Limit how much resources are used for view updates - Partial Update Engine & Object builder 
- Filtering(SQL counterparts): 
    - By relation – object builder (JOIN) 
    - By object id – object builder (WHERE) 
    - By schema id – schema registry (view definition) (FROM) 
    - By custom field – object builder (WHERE) 
    - By grouped field – materializers (HAVING) 
- Grouping, window functions – materializers 
- Joins – Object builder 
- Data transformations - materializers 

## Message Flow

Following diagram presents message flow in cdl related to materialized view functionality. Some components which do not participate in the process are omitted.   

![image (6)](https://user-images.githubusercontent.com/9082099/109971040-128d9600-7cf6-11eb-8a11-10e41334a757.png)


Data router receives new message: 
- When message is standard input data: 
	- Data router asks schema registry for the repository data needs to be inserted (or uses cache) (1)
    - Data router passes message to proper repository (2)
	- Repository inserts data into database and send notification to notification storage (e.g.  Kafka) (3)
- When message is information about relation between two objects: 
	- Data router passes message to Edge registry (4)
	- Edge registry sends notification to notification storage (5)


When there are new notifications in notification service (this will not happen right away; Partial Update Engine might be in sleep phase): 
- Partial Update Engine fetches information about changed objects and relations. It decides when materialized view needs to be partially recalculated (6)
- Partial Update Engine sends to object builder (8): 
    - view id 
    - array of ids on which we need to calculate the view(schema_id, object_id pairs)
- Object builder fetches view definition (or uses cache) (9)
- Object builder asks edge registry for relation graph for specified objects (10)
- Object builder fetches object information from repositories, performs filtering, join operations etc. (11) 
- Object builder sends transformed object data and array of ids to materializers (12)

## Schema repository 

Schema repository is responsible for storing information about schemas and views. Exact structure of view definition will be designed once we agree on general solution. 

## Partial Update Engine 

Partial Update Engine is responsible for decision on when partial view should be recalculated. It requests object builder to build partial view based on information of outdated records. 

Partial Update Engine Loop: 
- Fetch definition of all views (7; can use cache) 
- Process all pending notifications: 
	For each notification take every view definition and check if view data is affected by the change. Return list of object_id, schema_id pairs - called change list later in this document. 

	Notifications: 
    - Add relation between objects  
    - Delete relation between objects 
    - Insert/update object 

    If notification mentions object which can be used by view new record is added to the change list. For relation related notifications it is fine to add only one side of relation. 
- Run reduce on generated change lists – so there are no duplicates
- Split change lists if they are too large – max list size defined in config 
- Queue all views for which we generated change records to be refreshed 
- Go to sleep for X seconds – time defined in config 

## Object Builder 

Object builder responsibility is creating complex objects according to recipes - view definitions. 

Object Builder Loop:
- Wait for request to build a view 
- Fetch view definition from schema registry (9) 
- Fetch relation graph from Edge registry (10) 
- Fetch objects from repositories (11) 
- Perform filtering by custom fields, join operations 
- Send data to materializers component 

It is important to note that object builder output contains view id, change list received from partial update engine, and requested objects with information how they were created (each returned object contains ids of every object which was used for its creation). 

## Materializers

![image (5)](https://user-images.githubusercontent.com/9082099/109971136-2f29ce00-7cf6-11eb-8e61-bbc018f83f15.png)

There are two approaches on how materializers can look like. The main difference between them is if we allow grouping, window functions, filtering on grouped entities to happen in external services (e.g., Elasticsearch, Postgres). Problem with grouping is that merging multiple rows into one makes it almost impossible to apply partial updates.  

### Approach I 

Materializer is just a single connector service and a database. Its job is to update the database on changes received from object builder in a transactional way: delete all records based on objects from change list, add records returned from object builder. 

Grouping, advanced filtering and similar operations are done on the fly as user requests the data – they are mostly cheap operations. If that's not enough users can always create pipelines to do more data transformations. 

### Approach II 

This approach requires us to store another copy of data. First phase materializer is a component which extends the job of materializers from Approach I, but it uses storage which is internal to CDL. Then it performs data transformation (grouping, window functions, filtering on grouped records) on whole view (not just part of it) and pushes result down the pipeline to another  service which recreates the views in external storage. 

## Final notes

### What about fetching view data which are not materialized (views calculated on the fly)? 

Adding another specialized materializer solves this case easily. However, since message flow is bit different in this case (processing request starts from this specialized materializer) this case is not described in this document to not obscure general idea. 

## Alternative solutions

### Partial Update Engine periodically querying repositories for updated records
This approach should work correctly, but it has some minor drawback: 
- Even if cdl is idle (not data changes) partial update engine needs to do its work 
- Requires repositories to store last changed time which currently is not a requirement 

Anyway, those are minor drawback, we could as well use this solution. 

# RFC Changelog

* `19.03.2021` - Followed rename from `edge-repository` to `edge-registry`
