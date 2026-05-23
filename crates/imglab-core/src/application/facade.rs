pub struct ImgLabApplication<L, LL, A, G, M, AL, GA, S, T> {
    library: L,
    library_lifecycle: LL,
    assets: A,
    generation: G,
    metadata_review: M,
    albums: AL,
    gallery: GA,
    search: S,
    tasks: T,
}

pub struct ImgLabApplicationParts<L, LL, A, G, M, AL, GA, S, T> {
    pub library: L,
    pub library_lifecycle: LL,
    pub assets: A,
    pub generation: G,
    pub metadata_review: M,
    pub albums: AL,
    pub gallery: GA,
    pub search: S,
    pub tasks: T,
}

impl<L, LL, A, G, M, AL, GA, S, T> ImgLabApplication<L, LL, A, G, M, AL, GA, S, T> {
    pub fn from_parts(parts: ImgLabApplicationParts<L, LL, A, G, M, AL, GA, S, T>) -> Self {
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

    pub fn into_parts(self) -> ImgLabApplicationParts<L, LL, A, G, M, AL, GA, S, T> {
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
    }
}
