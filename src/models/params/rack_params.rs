pub struct CreateRackParams<'a> {
    pub warehouse_id: i64,
    pub code: &'a str,
    pub zone: Option<&'a str>,
    pub level: Option<i32>,
    pub capacity: Option<i64>,
    pub description: Option<&'a str>
}

pub struct UpdateRackParams<'a> {
    pub code: Option<&'a str>,
    pub zone: Option<&'a str>,
    pub level: Option<i32>,
    pub capacity: Option<i64>,
    pub description: Option<&'a str>
}

impl<'a> UpdateRackParams<'a> {
    pub fn is_empty(&self) -> bool {
        self.code.is_none() 
            && self.zone.is_none()
            && self.level.is_none()
            && self.capacity.is_none()
            && self.description.is_none()
    }
}