# narrative engine

structured context framework + agentic middleware

**BLOCKS**

the digestible units of context
(story, scene, snippet, etc.)

- narrative engine works over a sequential dataset
with indexed entries (postgres, NOSQL, etc.)

- retrieves a configurable, harmonized sequence
of blocks.
(recent-weighted, notable = true for noteworthy blocks
(i.e., exposition, notable events,
impactful changes))

Example:
```text
Legend:
  □ = block
  ■ = retrieved block
  ◇ = notable block


                           ◇           ◇                 ◇
                           │           │                 │
Story history
(oldest) ──►  ■  □  □  ■  □  □  ■  □  □  □  ■  ■  □  ■  ■  ■  ──► (newest)
              │        │        │           │  │     │  │  │
              └────────┴────────┴───────────┴──┴─────┴──┴──┘
                              retrieved context
```  

**PROVIDER**

a configurable **provider** leverages method contracts:
generate_block
generate_blocks_batch


**operation + inference**

an AI application utilizes narrative engine
as a middleware, providing its own
generative AI control:

**API / flow diagram**

```text
[ DATASTORE ]
     │
     └── 1. historical context ──► [ NARRATIVE ENGINE ]
                                      │
                                      ├── 2. retrieve context ──► [ PX ]
                                      │                              │
                                      │◄──────── context ────────────┘
                                      │
                                      └── 3. generate ──► [ NEW BLOCK ]
                                                            │
                                                            ├── 4. insert ──► [ DATASTORE ]
                                                            │
                                                            └── 5. return block + context payload ──► [ APPLICATION ]
```
                                                            
**2. retrieve context**

retrieval can be enhanced
by a secondary stage that provides multimodal data (px)
in the return envelope

essentially a middleware-inside-middleware:

```text
[ HISTORICAL CONTEXT ]
          │
          └──────────────► [ AI / AGENT SCANS FOR ]
                              │
                              ├─ entities
                              ├─ characters
                              ├─ locations
                              ├─ events
                              └─ props
                                      │
                                      └──────────────► [ RESULTS ]
                                                          │
                                                          ├─ entities with descriptions
                                                          ├─ representations
                                                          ├─ references
                                                          ├─ relationships
                                                          └─ event history
```

**3. generate block**

the result is a structured context
and entity map on each turn.

this structured context is passed
directly to the model to generate
the latest block -- informed by 
a rich historical and 
representational payload.

```text
[ ENTITY ]
    │
    ▼
[ IMAGE REPRESENTATIONS ]
    │  (signed URLs)
    ▼
[ RETURN ENVELOPE ]
    │
    ├─ block: string
    └─ representations[]
         {
           format: string
           uri: string
           name: string
           entityName: string
           id: string
           description?: string
         }
```

**parameters:**

parameters:
* max_unique_entity_representations
  (the number of unique entities to
  return images from.)

* block
  (latest of chronological blocks)
  
* representation property?
  (the name of the field to return, eg. avatar,
  character sheet, image, etc.) —
  **with fallbacks?**


* chronological_blocks
  (retrieved historical blocks in descending
  date order).
