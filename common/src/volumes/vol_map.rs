// Standard
use std::collections::HashMap;
use std::sync::Arc;

// Library
use vek::*;

// Crate
use crate::{
    vol::{BaseVol, ReadVol, SampleVol, SizedVol, VolSize, Vox, WriteVol},
    volumes::{
        chunk::{Chunk, ChunkErr},
        dyna::{Dyna, DynaErr},
    },
};

#[derive(Debug)]
pub enum VolMapErr {
    NoSuchChunk,
    ChunkErr(ChunkErr),
    DynaErr(DynaErr),
}

// V = Voxel
// S = Size (replace with a const when const generics is a thing)
// M = Chunk metadata
pub struct VolMap<V: Vox, S: VolSize, M> {
    pub chunks: HashMap<Vec3<i32>, Arc<Chunk<V, S, M>>>,
}

impl<V: Vox, S: VolSize, M> VolMap<V, S, M> {
    #[inline(always)]
    pub fn chunk_key(pos: Vec3<i32>) -> Vec3<i32> {
        pos.map2(S::SIZE, |e, sz| e.div_euclid(sz as i32))
    }

    #[inline(always)]
    pub fn chunk_offs(pos: Vec3<i32>) -> Vec3<i32> {
        pos.map2(S::SIZE, |e, sz| e.rem_euclid(sz as i32))
    }
}

impl<V: Vox, S: VolSize, M> BaseVol for VolMap<V, S, M> {
    type Vox = V;
    type Err = VolMapErr;
}

impl<V: Vox + Clone, S: VolSize, M> SampleVol for VolMap<V, S, M> {
    type Sample = Dyna<V, ()>;

    /// Take a sample of the terrain by cloning the voxels within the provided range.
    ///
    /// Note that the resultant volume does not carry forward metadata from the original chunks.
    fn sample(&self, range: Aabb<i32>) -> Result<Self::Sample, VolMapErr> {
        unimplemented!()
        // Return early if we don't have all the needed chunks that we need!
        /*
        let min_chunk = Self::chunk_key(range.min);
        let max_chunk = Self::chunk_key(range.max - Vec3::one());
        for x in min_chunk.x..=max_chunk.x {
            for y in min_chunk.y..=max_chunk.y {
                for z in min_chunk.z..=max_chunk.z {
                    if self.chunks.get(&Vec3::new(x, y, z)).is_none() {
                        return Err(VolMapErr::NoSuchChunk);
                    }
                }
            }
        }

        let mut sample = Dyna::filled(range.size().map(|e| e as u32).into(), V::empty(), ());

        let mut last_chunk_pos = self.pos_key(range.min);
        let mut last_chunk = self.get_key(last_chunk_pos);

        for pos in sample.iter_positions() {
            let new_chunk_pos = self.pos_key(range.min + pos);
            if last_chunk_pos != new_chunk_pos {
                last_chunk = self.get_key(new_chunk_pos);
                last_chunk_pos = new_chunk_pos;
            }
            sample
                .set(
                    pos,
                    if let Some(chunk) = last_chunk {
                        chunk
                            .get(Self::chunk_offs(range.min + pos))
                            .unwrap_or(V::empty())
                    // Fallback in case the chunk doesn't exist
                    } else {
                        self.get(range.min + pos)
                            .map(|v| v.clone())
                            .unwrap_or(V::empty())
                    },
                )
                .map_err(|err| VolMapErr::DynaErr(err))?;
        }

        Ok(sample)*/
    }
}

impl<'a, V: Vox + Clone, S: VolSize + Clone, M: Clone> ReadVol for VolMap<V, S, M> {
    #[inline(always)]
    fn get(&self, pos: Vec3<i32>) -> Result<V, VolMapErr> {
        let ck = Self::chunk_key(pos);
        self.chunks
            .get(&ck)
            .ok_or(VolMapErr::NoSuchChunk)?
            .get(Self::chunk_offs(pos))
            .map(|v| v.clone())
            .map_err(|err| VolMapErr::ChunkErr(err))
    }
}

impl<V: Vox + Clone, S: VolSize + Clone, M: Clone> WriteVol for VolMap<V, S, M> {
    #[inline(always)]
    fn set(&mut self, pos: Vec3<i32>, vox: V) -> Result<(), VolMapErr> {
        let ck = Self::chunk_key(pos);
        Arc::make_mut(self.chunks.get_mut(&ck)
                .ok_or(VolMapErr::NoSuchChunk)?
            )
            .set(Self::chunk_offs(pos), vox)
            .map_err(|err| VolMapErr::ChunkErr(err));

        Ok(())
    }
}

impl<V: Vox + Clone, S: VolSize + Clone, M: Clone> VolMap<V, S, M> {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
        }
    }

    pub fn chunk_size() -> Vec3<u32> {
        S::SIZE
    }

    pub fn insert(&mut self, key: Vec3<i32>, chunk: Chunk<V, S, M>) -> Option<Arc<Chunk<V, S, M>>> {
        self.chunks.insert(key, Arc::new(chunk))
    }

    pub fn get_key(&self, key: Vec3<i32>) -> Option<Arc<Chunk<V, S, M>>> {
        self.chunks.get(&key).map(|a| a.clone())
    }

    pub fn remove(&mut self, key: Vec3<i32>) -> Option<Arc<Chunk<V, S, M>>> {
        self.chunks.remove(&key)
    }

    pub fn key_pos(&self, key: Vec3<i32>) -> Vec3<i32> {
        key * S::SIZE.map(|e| e as i32)
    }

    pub fn pos_key(&self, pos: Vec3<i32>) -> Vec3<i32> {
        Self::chunk_key(pos)
    }

    pub fn iter<'a>(&'a self) -> ChunkIter<'a, V, S, M> {
        ChunkIter {
            iter: self.chunks.iter(),
        }
    }
}

pub struct ChunkIter<'a, V: Vox, S: VolSize, M> {
    iter: std::collections::hash_map::Iter<'a, Vec3<i32>, Arc<Chunk<V, S, M>>>,
}

impl<'a, V: Vox, S: VolSize, M> Iterator for ChunkIter<'a, V, S, M> {
    type Item = (Vec3<i32>, &'a Arc<Chunk<V, S, M>>);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(k, c)| (*k, c))
    }
}
