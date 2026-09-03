1 narrativeengine uses an application-provisioned datastore to retrieve historical blocks, lore, and search results. 
This retrieval can all be moved directly into durable objects -- blocks, lore, and even vector embeddings can live in an object's datastore.
As a result, history, episodic data, events, etc, can be owned by the same nap repository where entities and items live. 
Applications won't need to provision databases (or providers) -- they'll simply call narrative engine and provide the branch-scoped channelId.

