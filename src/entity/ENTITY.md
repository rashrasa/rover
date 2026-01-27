# Entity

## Description

Concrete types that implement any combination of the traits defined in `src/entity.rs`. 
For each unique combination, a new entity class should be created. This mitigates the potential memory inefficiency and speed issues
that come with an ECS paradigm.

All concrete types will be stored contiguously in a dynamic array improving cache locality when
iterating over them. Additionally, each tick, every entity is only iterated over once, compared to the many queries that have to be made
when scaling up an ECS to multiple systems.

The only problem is flexibility, as for each concrete type, a new Vec has to be made, all necessary functions have to be called in the right order and place,
and each concrete type has to be checked with all existing ones to sort out any interactions (e.g., two types implementing physics need to have their states updated before any tick occurs).
