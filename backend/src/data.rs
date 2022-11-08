#[derive(Debug)]
pub struct Restaurant {
    pub id: u32,
    pub name: String,
    pub address: String,
    pub dishes: Vec<Dish>,
}

#[derive(Debug)]
pub struct Dish {
    pub name: String,
    /// path to image
    pub image: String,
    pub review: Vec<Review>
}

#[derive(Debug)]
pub struct Review {
    pub reviewer: String,
    pub star: u8,
    pub comment: String,
}
