use cedar_policy::Entity;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Principal {
    entity: Entity,
    schema_id: String,
}

#[allow(dead_code)]
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
