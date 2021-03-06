use specs::prelude::*;
use super::{Viewshed, Position, Map, Player};
use rltk::{field_of_view, Point};

pub struct VisibilitySystem {}

impl<'a> System<'a> for VisibilitySystem {
    type SystemData = ( WriteExpect<'a, Map>,
                        Entities<'a>,
                        WriteStorage<'a, Viewshed>,
                        WriteStorage<'a, Position>,
                        ReadStorage<'a, Player>);

    fn run(&mut self, data : Self::SystemData) {
        let (mut map, entities, mut viewshed, pos, player) = data;

        for (_entitiy, _viewshed, _pos) in (&entities, &mut viewshed, &pos).join(){
            if _viewshed.dirty {
                _viewshed.dirty = false;
                _viewshed.visible_tiles.clear();
                _viewshed.visible_tiles = field_of_view(Point::new(_pos.x, _pos.y), _viewshed.range, &*map);
                _viewshed.visible_tiles.retain(|p| p.x >= 0 && p.x < map.width && p.y >= 0 && p.y < map.height );

                let _player : Option<&Player> = player.get(_entitiy);
                if let Some(_player) = _player {
                    for tile in map.visible_tiles.iter_mut() { *tile = false };
                    for vis in _viewshed.visible_tiles.iter() {
                        let idx = map.xy_idx(vis.x, vis.y);
                        map.revealed_tiles[idx] = true;
                        map.visible_tiles[idx] = true
                    }
                }
            }
        }
    }
}