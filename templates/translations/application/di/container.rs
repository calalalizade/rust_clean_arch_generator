use crate::features::translations::{
    application::interactor::i_translations_interactor::ITranslationsInteractor,
    domain::interactor::translations_interactor_impl::TranslationsInteractorImpl,
    infrastructure::{
        data_access::translations_data_source::TranslationsDataSource,
        repository::translations_repository_impl::TranslationsRepositoryImpl,
    },
};

pub struct TranslationsContainer;

impl TranslationsContainer {
    pub fn interactor() -> Box<dyn ITranslationsInteractor> {
        let data_source = Box::new(TranslationsDataSource::new());
        let repository = Box::new(TranslationsRepositoryImpl::new(data_source));
        Box::new(TranslationsInteractorImpl::new(repository))
    }
}
