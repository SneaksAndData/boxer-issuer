use cedar_policy::Entity;

#[derive(Debug, Clone)]
pub struct Principal {
    entity: Entity,
    schema_id: String,
}

impl Principal {
    pub fn new(entity: Entity, schema_id: String) -> Self {
        Self { entity, schema_id }
    }

    pub fn get_entity(&self) -> &Entity {
        &self.entity
    }

    pub fn get_schema_id(&self) -> &String {
        &self.schema_id
    }
}
