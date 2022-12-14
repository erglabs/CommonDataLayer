# Front Matter

```
    Title           : Edge Registry
    Author(s)       : kononnable
    Team            : CommonDataLayer
    Reviewer        : CommonDataLayer
    Created         : 2021-06-30
    Last updated    : 2021-06-30
    Category        : Feature
    CDL Feature ID  : CDLF-00012-00
```

# Edge registry

## Motivation 
CDL lacks the ability to declare relations between two objects. We need some way of storing that information if we want to be able to create complex views. Edge registry would allow us to store relations between objects no matter if they are stored in same repository or in different ones. 

## Proposed Changes
***
:memo: Proposed names are not final, and they are likely to be changed. They are here to provide the idea of what should be done, not exact way how it should be done nor how it should be named. 
***

Add a special kind of repository named edge registry. There will be at most one edge registry deployed at any time. It will keep information about relations on the object level and on the schema level. Schema registry should publish notifications for any object related change. 

### API
- Add relation type – adds schema level entry 
- Get relation type – returns schema level entry
- Delete relation type – removes schema level entry 
- List relation types – lists relations, can be filtered by schema id parameter 
- Add related object – adds object level entry 
- Get related objects – returns object level entries, filtered by relation_id and/or object_uuid list, requires at least one filter(should not be able to  query whole db at once) 
- Delete related – objects – removes object level entry 

### Information to be Stored

#### Schema Level
- relation id: UUID, autogenerated 
- schema 1 id: UUID 
- schema 2 id: UUID

#### Object Level
- relation id: UUID 
- object 1: UUID 
- object 2: UUID 

### Enhancements
Functionalities which might be worth implementing, but aren't required for the feature to be ready.

#### Storing Relations for Subobjects
Storing relationship between two flat structures is easy. Doing the same for nested objects is much more complicated. In case of nested object any child, grandchild etc. of the entity can be related to any other entity or one of its children. This makes storing relations outside of related object almost impossible. For example, we cannot use array indices – removal of one child would create a cascade effect. 

This can be migrated by introducing relation markers. Relation marker is an additional UUID field which is put directly to sub object which is related to another object. By adding those relation markers to object level entities, we can store information on sub object relationships without having to worry about object changes (from relation perspective). 

##### Required Changes
- Add Schema 1 Marker Path, Schema 2 Marker Path to Schema level – optional string fields containing information on where in object structure marker field is located (if this is a sub object relation) 
- Add Object 1 Marker, Object 2 Marker to Object level – optional UUID fields, marker values of related sub objects 

#### Relation Graph
Main advantage of keeping relation information separately from data is the ability to generate whole relation graph without touching the data itself. This makes generating relation graph a cheap operation and allows multiple optimizations to take place when data is being fetched. 

##### Optimizations
Since optimizations are just causally related to graph repository itself, they will only be briefly mentioned here. For more information on them see descriptions on how query plans are generated in SQL databases since those two topics (and their optimizations) are very much alike. 

Most SQL databases uses three implementations of join operation (hash join, merge join, nested loop). Proper algorithm is chosen based on amount of estimated number of rows affected by the operation (both inputs and output) and few other factors. Additionally, query engine decides which input set should be left and which should be right operand of the operation. Choosing wrong operand side can kill performance of some join algorithms. Another optimization that SQL Engines do is choosing proper order of join operations. When we expect multiple tables to be joined together its SQL Engine job to decide order result sets will be joined. The most performant way in most cases is just to sort trees of join operations by size of expected outcome. 

All mentioned optimizations are based on number of expected records used in join operation. Some of them requires knowledge of whole join graph before they can be used. 

##### Required Changes
- Add another api endpoint which for given input - object representing multiple join operations (relation tree) and array(s) of object ids – returns output – a tree containing object ids. 

##### Example
***
:memo: Input/output formats will be changed, for sake of convenience relations are represented by name (not UUID), object_ids as integer numbers. 
***
***
:memo: Filtering on edge registry layer can be done only for object_id fields. 
***

Get me all the employees, their addresses, information about their boss and boss's boss where employee id is one of 203, 403 and boss id is one of 2,4. 

SQL pseudocode: `Select * from employee join address join employee as boss join employee as boss2 where employee.id in [203,403] AND boss.id in [2,4]`

Input format: 
```
{ 
    root:  employee 
    relations: { 
        lives_in:  true, 
        boss: { 
            boss: true 
        } 
    }, 
    ids: [203,403],
    filters: { 
        boss: [2,4] 
    } 
} 
```
Output format: 
```
[
    {
        id: 203, 
        lives_in: [24], 
        boss: [
            { 
                id: 2, 
                boss: [1] 
            }
        ]
    }
] 
```
 

## Other Solutions

### Keep Relationship Information Directly on One/Both of Related Options
In this case to do any join operation we need to fetch all the values of specific schema. This can be costly for situations where only few items are related and join result is smaller than both inputs of join operation. Additionally, with this solution we cannot do optimizations mentioned in Relation graph section, or we can do them in a limited way. 

## More Things to Consider
- Should we check if object exists in registry before accepting relation data to be saved – technically it is simple to do (if we do not have object delete operation), however in async environment (message queues) it might be harder to use by clients – there is no message processing ordering guarantee between different repositories. 
- What should we do when user tries to remove relation on schema level if there are object level entries for this relation? 
- Should we have some mechanism of cleaning edge registry from entries to objects which do not exist 

# RFC Changelog

* `19.03.2021` - Changed name from `edge-repository` to `edge-registry`
