#  Front Matter

```
    Title           : CDL Ingestion API versioning - changes to the format
    Author(s)       : Mateusz 'esavier' Matejuk
    Team            : CommonDataLayer
    Reviewer        : CommonDataLayer
    Created         : 2021-06-14
    Last updated    : 2021-06-14
    Category        : Feature
    CDL feature ID  : CDLF-00010-00
```

#### Abstract

```
     The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL
     NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED",  "MAY", and
     "OPTIONAL" in this document are to be interpreted as described in
     RFC 2119.
```
[RFC 2119 source][rfc2119]

## Introduction
Due to outside requirements, we would like to introduce input protocol versioning.
This document states changes that needs to be made in CDL Ingestion Message format, and explains the reasoning behind those.

##### Formats
v1 CDL Input Message - as for today, this is the message format used as CDL's input.
```
{
  object_id: UUID,
  schema_id: UUID,
  data: { any valid json },
}
```

V1 CDL Input Message Change - proposed change related to versioning:
```
{
  version: String
  object_id: UUID,
  schema_id: UUID,
  data: { any valid json },
}
 ```

Format changes have to encompass changes related to other features if those exist.

## Changes to Data Router Behavior
Due to forementioned changes, DataRouter will have to discard messages that don't version field, as it is mandatory. Additionally, protocols that are not supported have to be discarded as well and error is to be emitted to appropriate channels.

#### Reasoning
It was requested for this feature to be as simple as possible, so we decided that stringified version will be used, i.e. "1.0". This will require additional complexity on implementation side but, will be easier to use by external systems.

### Notes
Currently, there is no process of determining if DataRouter can handle all the features provided by protocol version, however, protocol validation will be introduced as a new feature, and will undergo separate research and design, as validation is outside the scope of this document.

This feature must be revisited when encryption and compression of incoming streams are going to be implemented.

#####  References
Parent RFC:
[Version 1.0](archive/0005_CDL_Ingestion_API_versioning_01.md)

CDL :
[CDL project](https://github.com/epiphany-platform/CommonDataLayer)
[CDL - RFC discussions](https://github.com/epiphany-platform/CommonDataLayer/discussions/categories/rfc)
[CDL - RFC candidates](https://github.com/epiphany-platform/CommonDataLayer/tree/develop/docs/rfc)
[CDL - RFC releases](https://github.com/epiphany-platform/CommonDataLayer/tree/main/docs/rfc)

[rfc2119]:https://www.ietf.org/rfc/rfc2119.txt
[cdl-project]:https://github.com/epiphany-platform/CommonDataLayer
