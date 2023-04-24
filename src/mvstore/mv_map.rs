use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use anyhow::Result;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicI64, AtomicPtr};
use crate::{get_ref, get_ref_mut};
use crate::h2_rust_common::{Integer, Long, Nullable};
use crate::h2_rust_common::h2_rust_cell::H2RustCell;
use crate::h2_rust_common::h2_rust_type::H2RustType;
use crate::h2_rust_common::Nullable::NotNull;
use crate::mvstore::mv_store::MVStoreRef;
use crate::mvstore::page::{Page, PageTrait, PageTraitRef};
use crate::mvstore::r#type::data_type::DataType;
use crate::mvstore::root_reference::{RootReference, RootReferenceRef};

pub type MVMapRef = Option<Arc<H2RustCell<MVMap>>>;

#[derive(Default)]
pub struct MVMap {
    mv_store: MVStoreRef,
    id: Integer,
    create_version: Long,
    keys_buffer: Option<Vec<H2RustType>>,
    values_buffer: Option<Vec<H2RustType>>,
    pub key_type: Option<Arc<dyn DataType + Send + Sync>>,
    pub value_type: Option<Arc<dyn DataType + Send + Sync>>,
    single_writer: bool,

    /// volatile 通过set函数中的mutex模拟
    root_reference: RootReferenceRef,
    root_reference_set_mutex: Mutex<()>,

    avg_key_size: Option<AtomicI64>,
    avg_val_size: Option<AtomicI64>,
    keys_per_page: Integer,
    pub is_volatile: bool,

}

impl MVMap where  {
    pub fn new(mv_store_ref: MVStoreRef,
               id: Integer,
               key_type: Arc<dyn DataType + Send + Sync>,
               value_type: Arc<dyn DataType + Send + Sync>) -> Result<MVMapRef> {
        let keys_per_page = get_ref!(mv_store_ref).keys_per_page;
        let current_version = get_ref!(mv_store_ref).get_current_version();
        let mv_map_ref = Self::new1(mv_store_ref.clone(),
                                    key_type,
                                    value_type,
                                    id,
                                    0,
                                    None,
                                    keys_per_page,
                                    false)?;
        let mv_map = get_ref_mut!(mv_map_ref);

        mv_map.set_initial_root(mv_map.create_empty_leaf(mv_map_ref.clone()), get_ref!(mv_store_ref).get_current_version());

        Ok(mv_map_ref.clone())
    }

    fn new1(mv_store: MVStoreRef,
            key_type: Arc<dyn DataType + Send + Sync>,
            value_type: Arc<dyn DataType + Send + Sync>,
            id: Integer,
            create_version: Long,
            root_reference: RootReferenceRef,
            keys_per_page: Integer,
            single_writer: bool) -> Result<MVMapRef> {
        let mut mv_map = MVMap::default();

        mv_map.mv_store = mv_store;
        mv_map.id = id;
        mv_map.create_version = create_version;
        mv_map.key_type = Some(key_type);
        mv_map.value_type = Some(value_type);
        mv_map.root_reference = root_reference;
        mv_map.keys_per_page = keys_per_page;
        mv_map.single_writer = single_writer;

        if single_writer {
            mv_map.keys_buffer = Some(mv_map.key_type.as_ref().unwrap().create_storage(keys_per_page));
            mv_map.values_buffer = Some(mv_map.value_type.as_ref().unwrap().create_storage(keys_per_page));
        }

        if mv_map.key_type.as_ref().unwrap().is_memory_estimation_allowed() {
            mv_map.avg_key_size = Some(AtomicI64::new(0));
        }

        if mv_map.value_type.as_ref().unwrap().is_memory_estimation_allowed() {
            mv_map.avg_val_size = Some(AtomicI64::new(0));
        }


        Ok(Some(Arc::new(H2RustCell::new(mv_map))))
    }


    fn create_empty_leaf(&self, this: MVMapRef) -> PageTraitRef {
        Page::create_empty_leaf(this)
    }

    pub fn is_persistent(&self) -> bool {
        return self.mv_store.is_some() && !self.is_volatile;
    }

    fn set_initial_root(&mut self, root_page: PageTraitRef, version: Long) {
        self.set_root_reference(RootReference::new(root_page, version));
    }

    fn set_root_reference(&mut self, root_reference: RootReferenceRef) {
        self.root_reference_set_mutex.lock().unwrap();
        self.root_reference = root_reference;
    }

    /// set the position of the root page.
    pub fn set_root_pos(&self, root_pos: Long, version: Long, this: MVMapRef) {
        let root: PageTraitRef = self.read_or_create_root_page(root_pos, this);
    }

    fn read_or_create_root_page(&self, root_pos: Long, this: MVMapRef) -> PageTraitRef {
        if root_pos == 0 {
            self.create_empty_leaf(this)
        } else {
            self.read_page(this, root_pos).unwrap()
        };
        todo!()
    }

    fn read_page(&self, this: MVMapRef, position: Long) -> Result<PageTraitRef> {
        get_ref_mut!(self.mv_store).read_page(this, position)
    }
}