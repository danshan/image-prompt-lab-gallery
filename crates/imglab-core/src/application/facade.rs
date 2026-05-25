pub struct ImgLabApplication<L, LL, A, G, M, AL, GA, S, T, P, SC> {
    library: L,
    library_lifecycle: LL,
    assets: A,
    generation: G,
    metadata_review: M,
    albums: AL,
    gallery: GA,
    search: S,
    tasks: T,
    prompts: P,
    schedules: SC,
}

pub struct ImgLabApplicationParts<L, LL, A, G, M, AL, GA, S, T, P, SC> {
    pub library: L,
    pub library_lifecycle: LL,
    pub assets: A,
    pub generation: G,
    pub metadata_review: M,
    pub albums: AL,
    pub gallery: GA,
    pub search: S,
    pub tasks: T,
    pub prompts: P,
    pub schedules: SC,
}

impl<L, LL, A, G, M, AL, GA, S, T, P, SC> ImgLabApplication<L, LL, A, G, M, AL, GA, S, T, P, SC> {
    pub fn from_parts(parts: ImgLabApplicationParts<L, LL, A, G, M, AL, GA, S, T, P, SC>) -> Self {
        Self {
            library: parts.library,
            library_lifecycle: parts.library_lifecycle,
            assets: parts.assets,
            generation: parts.generation,
            metadata_review: parts.metadata_review,
            albums: parts.albums,
            gallery: parts.gallery,
            search: parts.search,
            tasks: parts.tasks,
            prompts: parts.prompts,
            schedules: parts.schedules,
        }
    }

    pub fn library(&self) -> &L {
        &self.library
    }

    pub fn library_lifecycle(&self) -> &LL {
        &self.library_lifecycle
    }

    pub fn assets(&self) -> &A {
        &self.assets
    }

    pub fn generation(&self) -> &G {
        &self.generation
    }

    pub fn metadata_review(&self) -> &M {
        &self.metadata_review
    }

    pub fn albums(&self) -> &AL {
        &self.albums
    }

    pub fn gallery(&self) -> &GA {
        &self.gallery
    }

    pub fn search(&self) -> &S {
        &self.search
    }

    pub fn tasks(&self) -> &T {
        &self.tasks
    }

    pub fn prompts(&self) -> &P {
        &self.prompts
    }

    pub fn schedules(&self) -> &SC {
        &self.schedules
    }

    pub fn into_parts(self) -> ImgLabApplicationParts<L, LL, A, G, M, AL, GA, S, T, P, SC> {
        ImgLabApplicationParts {
            library: self.library,
            library_lifecycle: self.library_lifecycle,
            assets: self.assets,
            generation: self.generation,
            metadata_review: self.metadata_review,
            albums: self.albums,
            gallery: self.gallery,
            search: self.search,
            tasks: self.tasks,
            prompts: self.prompts,
            schedules: self.schedules,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn facade_exposes_all_application_owners() {
        let app = ImgLabApplication::from_parts(ImgLabApplicationParts {
            library: "library",
            library_lifecycle: "library_lifecycle",
            assets: "assets",
            generation: "generation",
            metadata_review: "metadata_review",
            albums: "albums",
            gallery: "gallery",
            search: "search",
            tasks: "tasks",
            prompts: "prompts",
            schedules: "schedules",
        });

        assert_eq!(*app.library(), "library");
        assert_eq!(*app.library_lifecycle(), "library_lifecycle");
        assert_eq!(*app.assets(), "assets");
        assert_eq!(*app.generation(), "generation");
        assert_eq!(*app.metadata_review(), "metadata_review");
        assert_eq!(*app.albums(), "albums");
        assert_eq!(*app.gallery(), "gallery");
        assert_eq!(*app.search(), "search");
        assert_eq!(*app.tasks(), "tasks");
        assert_eq!(*app.prompts(), "prompts");
        assert_eq!(*app.schedules(), "schedules");
    }
}
