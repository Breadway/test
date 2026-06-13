//! Embedded pixel-art textures built from string grids — no asset files needed.
//!
//! Palette:
//!   '.' — transparent
//!   'X' — near-black outline  [12, 12, 14]
//!   '1' — white highlight     [255, 255, 255]
//!   '2' — light gray          [200, 200, 200]
//!   '3' — mid gray            [140, 140, 140]
//!   '4' — dark gray           [75,  75,  80]
//!   '5' — shadow              [40,  40,  44]
//!
//! All sprites are grayscale so Sprite.color tints them freely —
//! the GPU multiplies texture × color, giving every enemy and character
//! a distinct look without separate art per color variant.

use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

pub struct PixelArtPlugin;

impl Plugin for PixelArtPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, load_pixel_art);
    }
}

#[derive(Resource)]
pub struct GameSprites {
    pub player: Handle<Image>,
    pub roach:  Handle<Image>,
    pub spider: Handle<Image>,
    pub wasp:   Handle<Image>,
    pub larvae: Handle<Image>,
    pub beetle: Handle<Image>,
    pub boss:   Handle<Image>,
    pub bee:    Handle<Image>,
    pub bullet: Handle<Image>,
    pub bolt:   Handle<Image>,
    pub web:    Handle<Image>,
    pub lore_doc: Handle<Image>,
    pub wall:   Handle<Image>,
    pub floor:  Handle<Image>,
    pub spark:  Handle<Image>,
}

fn load_pixel_art(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let mut add = |art: &[&str]| images.add(make_image(art));
    let s = GameSprites {
        player:   add(PLAYER),
        roach:    add(ROACH),
        spider:   add(SPIDER),
        wasp:     add(WASP),
        larvae:   add(LARVAE),
        beetle:   add(BEETLE),
        boss:     add(BOSS),
        bee:      add(BEE),
        bullet:   add(BULLET),
        bolt:     add(BOLT),
        web:      add(WEB),
        lore_doc: add(LORE_DOC),
        wall:     add(WALL_TILE),
        floor:    add(FLOOR_TILE),
        spark:    add(SPARK),
    };
    commands.insert_resource(s);
}

fn pixel(c: char) -> [u8; 4] {
    match c {
        'X' => [12,  12,  14,  255],
        '1' => [255, 255, 255, 255],
        '2' => [200, 200, 200, 255],
        '3' => [140, 140, 140, 255],
        '4' => [75,  75,  80,  255],
        '5' => [40,  40,  44,  255],
        _   => [0,   0,   0,   0  ],
    }
}

pub fn make_image(art: &[&str]) -> Image {
    let height = art.len() as u32;
    let width  = art[0].len() as u32;
    let mut data = Vec::with_capacity((width * height * 4) as usize);
    for (i, row) in art.iter().enumerate() {
        debug_assert_eq!(
            row.len() as u32, width,
            "row {i} is {} wide, expected {width}",
            row.len()
        );
        for c in row.chars() {
            data.extend_from_slice(&pixel(c));
        }
    }
    Image::new(
        Extent3d { width, height, depth_or_array_layers: 1 },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

// ── Sprites ─────────────────────────────────────────────────────────────────

/// Exterminator in a hazmat suit — top-down view.
/// 16×16, displayed at 28×28.  Color tint = character color.
const PLAYER: &[&str] = &[
    "......XXXX......",
    ".....X2332X.....",
    "....X233332X....",
    "....X3XXXX3X....",
    "....X3X11X3X....",
    "....X3XXXX3X....",
    ".....X3332X.....",
    "..XXX222222XXX..",
    ".X44X222222X44X.",
    ".X44X222222X44X.",
    ".X44X222222X44X.",
    ".XXXX222222XXXX.",
    "....X33XX33X....",
    "....X33X.X33X...",
    "....X33X.X33X...",
    "....XXXX.XXXX...",
];

/// Cockroach — top-down oval body, antennae, six stubby legs.
/// 16×12, displayed at 24×16.
const ROACH: &[&str] = &[
    "..X...........X.",
    "...X.........X..",
    "...XXXXXXXXXX...",
    "...X22222222X...",
    "...X21111112X...",
    "...X21111112X...",
    "X..X22222222X..X",
    "X..X22222222X..X",
    "X..X22222222X..X",
    "...X21111112X...",
    "...X22222222X...",
    "...XXXXXXXXXX...",
];

/// Spider — top-down, symmetric 8-leg spread.
/// 16×16, displayed at 22×22.
const SPIDER: &[&str] = &[
    "X.....XXXX.....X",
    ".X....X22X....X.",
    "..X...X22X...X..",
    "...XXXXXXXXXX...",
    "...X22222222X...",
    "...X21111112X...",
    "...X21111112X...",
    "...X22222222X...",
    "...X23333332X...",
    "...X22222222X...",
    "...XXXXXXXXXX...",
    "..X...X22X...X..",
    ".X....X22X....X.",
    "X.....XXXX.....X",
    "................",
    "................",
];

/// Wasp — wings out wide, striped abdomen, pointed stinger.
/// 16×14, displayed at 18×18.
const WASP: &[&str] = &[
    "..11......11....",
    ".1221....1221...",
    "1222211112222X..",
    "1222X2222X2221..",
    ".11X222222X11...",
    "..X22XXXX22X....",
    "..X2X2112X2X....",
    "...X211112X.....",
    "...X233332X.....",
    "...X211112X.....",
    "....X2332X......",
    ".....X22X.......",
    "......XX........",
    ".......X........",
];

/// Larvae — soft, chubby grub.
/// 12×8, displayed at 20×14.
const LARVAE: &[&str] = &[
    "....XXXX....",
    "..XX2222XX..",
    ".X22211222X.",
    "X222X11X222X",
    "X222X11X222X",
    ".X22211222X.",
    "..XX2222XX..",
    "....XXXX....",
];

/// Beetle — armoured carapace split down the middle, broad and heavy.
/// 16×14, displayed at 30×26.
const BEETLE: &[&str] = &[
    ".....XXXXXX.....",
    "...XX222222XX...",
    "..X2222XX2222X..",
    ".X22222XX22222X.",
    ".X22221XX12222X.",
    "X222211XX112222X",
    "X222211XX112222X",
    "X222211XX112222X",
    "X222211XX112222X",
    ".X22221XX12222X.",
    ".X22222XX22222X.",
    "..X2222XX2222X..",
    "...XX5X..X5XX...",
    "....X.X..X.X....",
];

/// Broodmother — giant crowned queen bug, mandibles, swollen abdomen.
/// 24×24, displayed at 64×64.
const BOSS: &[&str] = &[
    "....X....XXXXXX....X....",
    ".....X..X222222X..X.....",
    "......XX22222222XX......",
    ".....X2222XXXX2222X.....",
    ".....X222X1111X222X.....",
    "....X2222X1111X2222X....",
    "....X22222XXXX22222X....",
    "...XX222222222222222XX..",
    "..X2X22222222222222X2X..",
    ".X22X22222222222222X22X.",
    "X2X222222222222222222X2X",
    "X2X2222233333332222222X2",
    "XX22222333113332222222XX",
    ".X22222331111332222222X.",
    ".X22222331111332222222X.",
    "XX22222333113332222222XX",
    "X2X2222333333332222222X2",
    "X2X222222222222222222X2X",
    ".X22X22222222222222X22X.",
    "..X2X22222222222222X2X..",
    "...X2X22222222222X2X....",
    "....X2X.X22222X.X2X.....",
    ".....X.X..XXXX..X.X.....",
    ".....X.X..........X.X...",
];

/// Ally bee — striped little buddy with transparent wings.
/// 8×8, displayed at 10×10.
const BEE: &[&str] = &[
    ".11..11.",
    "12211221",
    ".XXXXXX.",
    "X213312X",
    "X211112X",
    "X233332X",
    ".X2332X.",
    "..XXXX..",
];

/// Player projectile — glowing oval shot.
/// 8×8, displayed at 8×8, tinted by character color.
const BULLET: &[&str] = &[
    "...XX...",
    "..X22X..",
    ".X2112X.",
    "X211112X",
    "X211112X",
    ".X2112X.",
    "..X22X..",
    "...XX...",
];

/// Enemy projectile — jagged energy bolt.
/// 8×8, displayed at 8×8, tinted by enemy color.
const BOLT: &[&str] = &[
    "...XX...",
    "..X33X..",
    ".X3113X.",
    "X311113X",
    "X311113X",
    ".X3113X.",
    "..X33X..",
    "...XX...",
];

/// Web glob — radial cross with diagonals.
/// 10×10, displayed at 10×10.
const WEB: &[&str] = &[
    "1...X...1.",
    ".1..X..1..",
    "..1.X.1...",
    "...1X1....",
    "XXXXX.XXXX",
    "...1X1....",
    "..1.X.1...",
    ".1..X..1..",
    "1...X...1.",
    "..........",
];

/// Confidential BMG memo — folded top-right corner, redacted lines.
/// 12×14, displayed at 16×20.
const LORE_DOC: &[&str] = &[
    "XXXXXXXXX2X.",
    "X22222222X2X",
    "X2111111111X",
    "X2133333331X",
    "X2111111111X",
    "X2133333331X",
    "X2111111111X",
    "X2133311111X",
    "X2111111111X",
    "X2133333331X",
    "X2111111111X",
    "X2111111111X",
    "X2111111111X",
    "XXXXXXXXXXXX",
];

/// Riveted metal wall plating — tile at 16px.
const WALL_TILE: &[&str] = &[
    "5555555555555555",
    "5445444444445445",
    "5454444444445445",
    "5444444444444445",
    "5444444444444445",
    "5444444444444445",
    "5444444444444445",
    "5555555555555555",
    "5555555555555555",
    "5444444444444445",
    "5444444444444445",
    "5444444444444445",
    "5444444444444445",
    "5454444444445445",
    "5445444444445445",
    "5555555555555555",
];

/// Dark facility floor grate — tile at 16px.
const FLOOR_TILE: &[&str] = &[
    "5555555555555555",
    "5XXXXXXXXXXXXXX5",
    "5X4444444444445X",
    "5X4444444444445X",
    "5X4444444444445X",
    "5X4444444444445X",
    "5X4444444444445X",
    "5X4444444444445X",
    "5X4444444444445X",
    "5X4444444444445X",
    "5X4444444444445X",
    "5X4444444444445X",
    "5X4444444444445X",
    "5X4444444444445X",
    "5XXXXXXXXXXXXXX5",
    "5555555555555555",
];

/// Impact spark — 4×4 bright pixel used for hit particles.
const SPARK: &[&str] = &[
    ".11.",
    "1111",
    "1111",
    ".11.",
];
