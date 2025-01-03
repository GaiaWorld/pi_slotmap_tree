use pi_null::Null;
use std::hash::Hash;
use pi_slotmap::{DefaultKey as DefaultKey1, Key, KeyData, SecondaryMap, SlotMap};

use crate::{Up, Down, Storage, StorageMut, Layer};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
pub struct TreeKey(DefaultKey1);

impl Key for TreeKey {
    fn data(&self) -> pi_slotmap::KeyData {
        self.0.data()
    }
	fn index(&self) -> usize {self.0.index()}
    
    fn with(idx: usize) -> Self {
        TreeKey(DefaultKey1::with(idx))
    }
}

impl From<KeyData> for TreeKey {
    fn from(data: KeyData) -> Self {
        Self(DefaultKey1::from(data))
    }
}

impl Null for TreeKey {
    fn null() -> Self {
        Self(DefaultKey1::null())
    }

    fn is_null(&self) -> bool {
		self.0.is_null()
    }
}

#[derive(Default)]
pub struct SlotMapTree {
	up: SecondaryMap<TreeKey, Up<TreeKey>>,
	down: SecondaryMap<TreeKey, Down<TreeKey>>,
	layer: SecondaryMap<TreeKey, Layer<TreeKey>>,
}

impl Storage<TreeKey> for SlotMapTree {
    fn get_up(&self, k: TreeKey) -> Option<&Up<TreeKey>> {
        self.up.get(k)
    }

    fn up(&self, k: TreeKey) -> &Up<TreeKey> {
        self.up.get(k).unwrap()
    }

    fn get_layer(&self, k: TreeKey) -> Option<&Layer<TreeKey>> {
        self.layer.get(k)
    }

    fn layer(&self, k: TreeKey) -> &Layer<TreeKey> {
        self.layer.get(k).unwrap()
    }

    fn get_down(&self, k: TreeKey) -> Option<&Down<TreeKey>> {
        self.down.get(k)
    }

    fn down(&self, k: TreeKey) -> &Down<TreeKey> {
        self.down.get(k).unwrap()
    }
}

impl StorageMut<TreeKey> for SlotMapTree {
    fn get_up_mut(&mut self, k: TreeKey) -> Option<&mut Up<TreeKey>> {
        self.up.get_mut(k)
    }

    fn up_mut(&mut self, k: TreeKey) -> &mut Up<TreeKey> {
        self.up.get_mut(k).unwrap()
    }

    fn get_down_mut(&mut self, k: TreeKey) -> Option<&mut Down<TreeKey>> {
        self.down.get_mut(k)
    }

    fn down_mut(&mut self, k: TreeKey) -> &mut Down<TreeKey> {
        self.down.get_mut(k).unwrap()
    }

    fn set_up(&mut self, k: TreeKey, parent: Up<TreeKey>) {
        self.up.insert(k, parent);
    }

    fn remove_up(&mut self, k: TreeKey) {
        self.up.remove(k);
    }

    fn set_layer(&mut self, k: TreeKey, layer: Layer<TreeKey>) {
        self.layer.insert(k, layer);
    }

    fn remove_layer(&mut self, k: TreeKey) {
		self.layer.remove(k);
    }

    fn set_down(&mut self, k: TreeKey, children: Down<TreeKey>) {
        self.down.insert(k, children);
    }

    fn remove_down(&mut self, k: TreeKey) {
        self.down.remove(k);
    }

    fn set_root(&mut self, _k: TreeKey) {
    }

    fn remove_root(&mut self, _k: TreeKey) {
    }
}

#[test]
fn test() {
    use crate::Tree;
    let mut tree: Tree<TreeKey, SlotMapTree> = Tree::new(SlotMapTree::default());

    let mut slotmap = SlotMap::default();
    let p1 = TreeKey(slotmap.insert(()));
    tree.insert_child(p1, TreeKey::null(), std::usize::MAX);

    
   

    let c1 = TreeKey(slotmap.insert(()));
    let c2 = TreeKey(slotmap.insert(()));
    let c3 = TreeKey(slotmap.insert(()));
    let c4 = TreeKey(slotmap.insert(()));
    let c5 = TreeKey(slotmap.insert(()));
    tree.insert_child(c4, p1, std::usize::MAX);
    tree.insert_child(c3, p1, std::usize::MAX);
    tree.insert_brother(c1, c4, crate::InsertType::Front);
    tree.insert_brother(c2, c4, crate::InsertType::Front);
    tree.insert_brother(c3, c4, crate::InsertType::Front);
    tree.insert_child(c5, p1, std::usize::MAX);

    println!("{:?}, {:?}, {:?}", c1, tree.get_storage().get_up(c1).unwrap().prev(), tree.get_storage().get_up(c1).unwrap().next());
    println!("{:?}, {:?}, {:?}", c2, tree.get_storage().get_up(c2).unwrap().prev(), tree.get_storage().get_up(c2).unwrap().next());
    println!("{:?}, {:?}, {:?}", c3, tree.get_storage().get_up(c3).unwrap().prev(), tree.get_storage().get_up(c3).unwrap().next());
    println!("{:?}, {:?}, {:?}", c4, tree.get_storage().get_up(c4).unwrap().prev(), tree.get_storage().get_up(c4).unwrap().next());
    println!("{:?}, {:?}, {:?}", c5, tree.get_storage().get_up(c5).unwrap().prev(), tree.get_storage().get_up(c5).unwrap().next());

}