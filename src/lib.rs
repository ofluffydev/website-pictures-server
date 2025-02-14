pub struct Gallery {
    pub images: Vec<Image>,
    pub highlight: Option<Image>,
}

pub struct Image {
    pub name: String,
    pub path: String,
}

impl Gallery {
    pub fn new() -> Gallery {
        Gallery {
            images: Vec::new(),
            highlight: None,
        }
    }

    pub fn add_image(&mut self, image: Image) {
        self.images.push(image);
    }

    pub fn set_highlight(&mut self, image: Image) {
        self.highlight = Some(image);
    }

    pub fn get_highlight(&self) -> Option<&Image> {
        self.highlight.as_ref()
    }
}