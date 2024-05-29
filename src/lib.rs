/// 树
/// 以链表结构维护一个树的子节点
/// 本模块只关心树中节点的插入、删除等逻辑部分，具体描述树状结构的节点数据由外部维护
pub mod slot_map_tree;

use core::panic;
use std::fmt::Debug;
use std::default::Default;
use std::ops::Deref;
use pi_print_any::out_any;

use serde::{Serialize, Deserialize};
pub use slot_map_tree::{SlotMapTree, TreeKey};


use pi_null::Null;

pub enum InsertType {
    Back,
    Front,
}

pub trait Storage<K: Null> {
	fn get_up(&self, k: K) -> Option<&Up<K>>;
	fn up(&self, k: K) -> &Up<K>;

	fn get_layer(&self, k: K) -> Option<&Layer<K>>;
	fn layer(&self, k: K) -> &Layer<K>;

	fn get_down(&self, k: K) -> Option<&Down<K>>;
	fn down(&self, k: K) -> &Down<K>;
}

pub trait StorageMut<K: Null>: Storage<K> {
	fn get_up_mut(&mut self, k: K) -> Option<&mut Up<K>>;
	fn set_up(&mut self, k: K, parent: Up<K>);
	fn up_mut(&mut self, k: K) -> &mut Up<K>;
	fn remove_up(&mut self, k: K);

	fn set_layer(&mut self, k: K, layer: Layer<K>);
	fn remove_layer(&mut self, k: K);

	fn get_down_mut(&mut self, k: K) -> Option<&mut Down<K>>;
	fn set_down(&mut self, k: K, children: Down<K>);
	fn down_mut(&mut self, k: K) -> &mut Down<K>;
	fn remove_down(&mut self, k: K);

	fn set_root(&mut self, k: K);
	fn remove_root(&mut self, k: K);
}

/// 父信息
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Up<K> {
	parent: K, // parent的索引
	prev: K, // 在父节点的子列表中，我的前一个节点
	next: K, // 在父节点的子列表中，我的后一个节点
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layer<K> {
	layer: usize,
	root: K,
}

impl<K: Clone + Copy> Layer<K>  {
	#[inline]
	pub fn layer(&self) -> usize{
		self.layer
	}
	#[inline]
	pub fn root(&self) -> K {
		self.root
	}
}

impl<K: Null> Default for Layer<K>{
	fn default() -> Self {
		Layer {
			layer: usize::null(),
			root: K::null(),
		}
	}
}

impl<K: Clone + Copy> Up<K>  {
	#[inline]
	pub fn new(id: K, prev: K, next: K) -> Self{
		Up {
			parent: id, prev, next
		}
	}
	#[inline]
	pub fn parent(&self) -> K {
		self.parent
	}
	#[inline]
	pub fn prev(&self) -> K {
		self.prev
	}
	#[inline]
	pub fn next(&self) -> K {
		self.next
	}
}

impl<K: Null> Default for Up<K> {
	fn default() -> Self {
		Up {
			parent: K::null(),
			prev: K::null(),
			next: K::null(),
		}
	}
}

/// 子信息
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Down<K> {
    pub head: K, // 子节点列表的的头节点
    pub tail: K, // 子节点列表的尾节点
    pub len: usize, // 子节点的长度
	pub count: usize, // 递归子节点的数量
}

impl<K: Clone + Copy> Down<K>  {
	#[inline]
	pub fn new(head: K, tail: K, len: usize, count: usize) -> Self {
		Down {
			head, tail, len, count
		}
	}

	#[inline]
	pub fn head(&self) -> K {
		self.head
	}
	#[inline]
	pub fn tail(&self) -> K {
		self.tail
	}
	#[inline]
	pub fn len(&self) -> usize {
		self.len
	}
	#[inline]
	pub fn count(&self) -> usize {
		self.count
	}
}

impl<K: Null> Default for Down<K> {
	fn default() -> Self {
		Down {
			head: K::null(),
			tail: K::null(),
			len: 0,
			count: 0,
		}
	}
}

pub struct Tree<K: Null, S> {
	storage: S,
	default_children: Down<K>,
}

impl<K: Null, S: Storage<K>> Deref for Tree<K, S> {
	type Target = S;

	fn deref(&self) -> &Self::Target {
		&self.storage
	}
}

// Down {
// 	head: K::null(),
// 	tail: K::null(),
// 	len: 0,
// 	count: 1,
// }

impl<K: Null + Eq + Clone + Copy, S> Tree<K, S> {
	pub fn new(storage: S) -> Self {
		Self {
			storage,
			default_children: Down { head: K::null(), tail: K::null(), len: 0, count: 1 },
		}
	}

	pub fn get_storage(&self) -> &S {
		&self.storage
	}
}

impl<K: Null + Eq + Clone + Copy, S: Storage<K>> Tree<K, S> {
	/// 迭代指定节点的所有子元素
	pub fn iter(&self, node_children_head: K) -> ChildrenIterator<K, S> {
		ChildrenIterator::new(&self.storage, node_children_head)
	}

	/// 迭代指定节点的所有递归子元素
	pub fn recursive_iter(&self, node_children_head: K) -> RecursiveIterator<K, S> {
		let (head, len) = if node_children_head.is_null() {
			(K::null(), 0)
		} else {
			(node_children_head, 1)
		};
		RecursiveIterator::new(&self.storage, head, len)
	}
}

impl<K: Null + Eq + Clone + Copy, S: StorageMut<K>> Tree<K, S> {

    /// index为0表示插入到子节点队列前， 如果index大于子节点队列长度，则插入到子节点队列最后。parent如果为0 表示设置为根节点。 如果parent的layer大于0
	/// order表示在子节点中的顺序，当大于子节点长度时，插入到队列最后
    pub fn insert_child(&mut self, id: K, parent: K, mut order: usize) {
		if cfg!(debug_assertions) {
			if id == parent {
				panic!("{:?}", pi_print_any::out_any!(format, "insert_child fail, id and parent is equal, id: {:?}, parent: {:?}", id, parent));
			}
		}

		pi_print_any::out_any!(log::debug, "insert_child, id={:?}, parent={:?}, order={:?}", id, parent, order);

        if !parent.is_null() {
            let (p_down, layer) = (
				// self.storage.get_parent(parent), 
				self.storage.get_down(parent).unwrap_or(&self.default_children),
				self.storage.get_layer(parent).map_or(Layer::default(), |layer|{ Layer {layer: if layer.layer.is_null() {usize::null()} else{ layer.layer + 1 }, root: layer.root}})
			);

			let (prev, next) = if order >= p_down.len {
				(p_down.tail, K::null())
			} else if order + order >= p_down.len {
				// 优化：order顺序在子节点列表中比较靠后，则从最后的位置开始向前寻找对应位置
				let mut prev = p_down.tail;
				let mut next = K::null();
				order = p_down.len - order;
				while order > 0 && !prev.is_null(){
					order -= 1;
					next = prev;
					prev = self.storage.get_up(next).unwrap().prev;
				}
				(prev, next)
			} else {
				// 优化：order顺序在子节点列表中比较靠前，则从第一个个位置开始向后寻找对应位置
				let mut prev = K::null();
				let mut next = p_down.head;
				while order > 0 && !next.is_null() {
					order -= 1;
					prev = next;
					next = self.storage.get_up(prev).unwrap().next;
				}
				(prev, next)
			};

            self.insert_node(id, parent, layer, prev, next);
        } else {
            self.insert_as_root(id)
        }
    }
    /// 根据InsertType插入到brother的前或后。 brother的layer大于0
    pub fn insert_brother(&mut self, id: K, brother: K, insert: InsertType) {
		pi_print_any::out_any!(log::debug, "insert_brother, id={:?}, brother={:?}, insert={:?}", id, brother, &insert);
        let (parent, layer, prev, next) = match (self.storage.get_up(brother), self.storage.get_layer(brother)) {
            (Some(up), layer) => match insert {
                InsertType::Front => (up.parent, layer.map_or(Layer::default(), |l|{l.clone()}), up.prev, brother),
                InsertType::Back => (up.parent, layer.map_or(Layer::default(), |l|{l.clone()}), brother, up.next),
            },
            _ => {
				out_any!(log::error, "invalid brother: {:?}", brother);
				panic!("")
			}
        };
		if cfg!(debug_assertions) {
			if id == parent {
				panic!("{:?}", pi_print_any::out_any!(format, "insert_brother fail, id and parent is equal, id: {:?}, parent: {:?}", id, parent));
			}
		}
        if !parent.is_null() {
            self.insert_node(id, parent, layer, prev, next)
        } else {
            self.insert_as_root(id)
        }
    }
    
    /// 从树上将节点移除（删除节点上的layer，并设置到正确的节点关联关系、子节点统计数量）
    pub fn remove(
        &mut self,
        id: K,
    ) {
		pi_print_any::out_any!(log::debug, "remove, id={:?}", id);
		// 删除所有递归子节点的layer
		if let Some(layer) = self.storage.get_layer(id) {
			if !layer.layer().is_null() {
				if layer.layer() == 1 {
					self.storage.remove_root(id);
				}
				self.remove_tree(self.storage.get_down(id).map_or(K::null(), |down|{down.head}));
			}
		}

		if let Some(up)  = self.storage.get_up_mut(id) {
			if !up.parent.is_null() {
				let (parent, prev, next) = (up.parent, up.prev, up.next);
				let count = self.storage.get_down(id).map_or(1, |down|{down.count + 1});
				self.remove_node(id, parent, count, prev, next);
			}
		}
	}

    // 插入节点, 如果id就在parent内则为调整位置
    fn insert_node(
        &mut self,
        id: K,
        parent: K,
        layer: Layer<K>,
        prev: K,
        next: K,
    ) {
		// // 调用该方法，该节点可能已经存在，并且是将该节点插入到原位置
		// // 如果插入到原位置，则无需操作
		// if id == prev || id == next {
		// 	return layer;
		// }

        let (count, fix_prev, fix_next) = match self.storage.get_up_mut(id) {
            Some(n) if !n.parent.is_null() => {
				// 当前插入节点已经有一个父节点，并且该节点的父节点与当前指定的兄弟节点的父节点不是同一个
				// 则panic
				if n.parent != parent {
					out_any!(log::error, "has a parent node, id: {:?}, old parent: {:?}, new_parent: {:?}", id, n.parent, parent);
					panic!("")
				}

				// 否则，当前节点存在一个父节点，则调整该节点的兄弟节点即可
				let fix_prev = n.prev;
				let fix_next = n.next;
				n.prev = prev;
				n.next = next;
				(0, fix_prev, fix_next)
            }
            _ => {
				// 不存在父节，直接挂在树上
				if !layer.layer.is_null() {
					self.storage.set_layer(id, layer.clone());
				}
				
				self.storage.set_up(id, Up {
					parent,
					prev,
					next,
				});
				self.storage.get_down(id).map_or((1, K::null(), K::null()), |c|{
					(c.count + 1, c.head, K::null())
				})
			},
		};
        // 修改prev和next的节点
        if !prev.is_null() {
            let mut node = self.storage.up(prev).clone();
			node.next = id;
			self.storage.set_up(prev, node);
        }
        if !next.is_null() {
            let mut node = self.storage.up(next).clone();
            node.prev = id;
			self.storage.set_up(next, node);
        }
        if count == 0 {
            // 同层调整
            if !fix_prev.is_null() {
                let mut node = self.storage.up(fix_prev).clone();
                node.next = fix_next;
				self.storage.set_up(fix_prev, node);
            }
            if !fix_next.is_null() {
                let mut node = self.storage.up(fix_next).clone();
                node.prev = fix_prev;
				self.storage.set_up(fix_next, node);
            }

            if prev.is_null() || next.is_null() || fix_prev.is_null() || fix_next.is_null() {
                let mut down = self.storage.down(parent).clone();
                if prev.is_null() {
                    down.head = id;
                } else if fix_prev.is_null() {
                    down.head = fix_next;
                }
                if next.is_null() {
                    down.tail = id;
                } else if fix_next.is_null() {
                    down.tail = fix_prev;
                }
            }
        }
		// 修改parent的children, count
		let mut p_down = self.storage.get_down(parent).map_or(Down::default(), |c|{c.clone()});
		if prev.is_null() {
			p_down.head = id;
		}
		if next.is_null() {
			p_down.tail = id;
		}
		p_down.len += 1;
		p_down.count += count;
		self.storage.set_down(parent, p_down);

		let p_p = self.storage.get_up(parent).map_or(K::null(), |p|{p.parent});
		// 递归向上修改count
		self.modify_count(p_p, count as isize);

		// layer.layer.is_null(), 并且不是同层调整时，才递归设置layer
        if !layer.layer.is_null() && count > 0 {
            self.insert_tree(fix_prev, Layer {layer: layer.layer + 1, root: layer.root.clone()});
			// 再次设置当前节点的layer，表明该节点是作为挂在主树上的一个子树的根
			self.storage.set_layer(id, layer);
		}
    }

	/// 创建一个根节点
	fn insert_as_root(&mut self, id: K) {
        // 设置为根节点
		match self.storage.get_up(id) {
			// 将节点作为根节点插入到树失败，节点已经存在一个父
			Some(up) if !up.parent.is_null() => {
				out_any!(log::error, "insert_root fail, node has a parent, id: {:?}, parent: {:?}", id, up.parent);
				return;
			},
			_ => {
				self.storage.set_root(id);
				self.storage.set_layer(id, Layer {layer: 1, root: id});
				let head = match self.storage.get_down(id) {
					Some(down) => down.head,
					None => {
						self.storage.set_down(id, Down {
							head: K::null(),
							tail: K::null(),
							len: 0,
							count: 0,
						});
						K::null()
					}
				};
				self.insert_tree(head, Layer {layer: 2, root: id});
				self.storage.set_layer(id, Layer {layer: 1, root: id}); // 设置第二遍，表明为子树的根
			},
		};
    }
	
    // 插入到树上， 就是递归设置每个子节点的layer
	// 安全：调用该方法，确保!layer.is_null()
    fn insert_tree(&mut self, mut id: K, layer: Layer<K>) {
        while !id.is_null() {
            let head = {
				self.storage.set_layer(id, layer.clone());
                let head = self.storage.get_down(id).map_or(K::null(), |down|{down.head});
				if let Some(up) = self.storage.get_up(id) {
					id = up.next;
				} else {
					id = K::null();
				}
				head
            };
            self.insert_tree(head, Layer {layer: layer.layer + 1, root: layer.root});
        }
    }
    // 从树上移除， 就是递归设置每个子节点, 删除layer
    fn remove_tree(&mut self, mut id: K) {
        while !id.is_null() {
            self.storage.remove_layer(id); // 删除layer

			if let Some(down) = self.storage.get_down(id) {
				// 如果存在子节点，则递归删除layer
				let head = down.head;
				self.remove_tree(head);
				id = self.storage.up(id).next;
			} else {
				break;
			}
		}
    }
    // // 递归销毁
    // fn recursive_destroy(&mut self, parent: K, mut id: K) {
	// 	self.storage.delete_children(parent);
    //     while !id.is_null() {
	// 		if let Some(down) = self.storage.get_children(id) {
	// 			self.recursive_destroy(id, down.head);
	// 		};
    //         id = self.storage.parent(id).next;
    //     }
    // }

    // 递归向上，修改节点的count
    fn modify_count(&mut self, mut id: K, count: isize) {
        while !id.is_null() {
			let down = self.storage.down_mut(id);
			down.count = (down.count as isize + count) as usize;
			// 除了修改，需要发通知吗？TODO
			if let Some(up) = self.storage.get_up_mut(id) {
				id = up.parent;
			} else {
				break;
			}
        }
    }
    // 移除节点
    fn remove_node(&mut self, id: K, parent: K, count: usize, prev: K, next: K) {
        // 修改prev和next的节点
        if !prev.is_null() {
            let node = self.storage.up_mut(prev);
			node.next = next;
        }
        if !next.is_null() {
            let node = self.storage.up_mut(next);
			node.prev = prev;
        }
        
		// 修改parent的children, count
		let p_down = self.storage.down_mut(parent) ;
		if prev.is_null() {
			p_down.head = next;
		}
		if next.is_null() {
			p_down.tail = prev;
		}
		p_down.len -= 1;
		p_down.count -= count;

		let p_p = self.storage.get_up(parent).map_or(K::null(), |up|{up.parent});
            

        // 递归向上修改count
		self.modify_count(p_p, -(count as isize));

		// 设置节点的层为None
		self.storage.remove_layer(id);

		// 设置up信息为null
		self.storage.remove_up(id);
    }
}

// pub struct ChildrenMutIterator<'a, K: Null, S: Storage<K>> {
//     inner: &'a mut S,
//     head: K,
// }
// impl<'a, K: Null, S: Storage<K>> Iterator for ChildrenMutIterator<'a, K, S> {
//     type Item = K;
//     fn next(&mut self) -> Option<Self::Item> {
// 		if self.head.is_null() {
// 			return None;
// 		}
// 		let head = self.head;

//         let inner = unsafe { &mut *(self.inner as *mut S) };
//         let n = unsafe { inner.get_unchecked_mut(head) };
//         let next = n.next;
//         let r = Some((head, n));
//         self.head = next;
//         r
//     }
// }
pub struct ChildrenIterator<'a, K: Null + Copy + Clone, S: Storage<K>>{
    inner: &'a S,
    head: K,
}

impl<'a, K: Null + Copy + Clone, S: Storage<K>> ChildrenIterator<'a, K, S> {
	pub fn new(s: &'a S, head: K) -> Self {
		ChildrenIterator {
			inner: s,
			head
		}
	}
}

impl<'a, K: Null + Copy + Clone, S: Storage<K>> Iterator for ChildrenIterator<'a, K, S> {
    type Item = K;

    fn next(&mut self) -> Option<Self::Item> {
		if self.head.is_null() {
			return None;
		}
		let r = self.head;
        match self.inner.get_up(self.head) {
			Some(up) => self.head = up.next,
			None => self.head = K::null(),
		};
        Some(r)
    }
}

pub struct RecursiveIterator<'a, K: Null, S: Storage<K>> {
    inner: &'a S,
    arr: [K; 32],
    len: usize,
}

impl<'a, K: Null + Copy + Clone, S: Storage<K>> RecursiveIterator<'a, K, S> {
	pub fn new(s: &'a S, head: K, len: usize) -> Self {
		RecursiveIterator {
			inner: s,
			arr: [
				head,
				K::null(),
				K::null(),
				K::null(),
				K::null(),
				K::null(),
				K::null(),
				K::null(),
				K::null(),
				K::null(),
				K::null(),
				K::null(),
				K::null(),
				K::null(),
				K::null(),
				K::null(),
				K::null(),
				K::null(),
				K::null(),
				K::null(),
				K::null(),
				K::null(),
				K::null(),
				K::null(),
				K::null(),
				K::null(),
				K::null(),
				K::null(),
				K::null(),
				K::null(),
				K::null(),
				K::null(),
			],
			len,
		}
	}
}

impl<'a, K: Null + Copy + Clone, S: Storage<K>> Iterator for RecursiveIterator<'a, K, S> {
    type Item = K;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        let head = self.arr[self.len];
		if let Some(up) = self.inner.get_up(head) {
			if !up.next.is_null() {
				self.arr[self.len] = up.next;
				self.len += 1;
			}
		}

		if let Some(down) = self.inner.get_down(head) {
			if !down.head.is_null(){
				self.arr[self.len] = down.head;
				self.len += 1;
			}
		};

        Some(head)
    }
}