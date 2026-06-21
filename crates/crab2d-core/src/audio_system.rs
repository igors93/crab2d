use std::collections::HashSet;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

pub struct AudioSystem {
    // rodio OutputStream must be kept alive for audio to play
    _stream: Option<rodio::OutputStream>,
    handle: Option<rodio::OutputStreamHandle>,
    playing: HashSet<String>,
    asset_roots: Vec<PathBuf>,
}

impl AudioSystem {
    pub fn new(asset_roots: Vec<PathBuf>) -> Self {
        let (stream, handle) = match rodio::OutputStream::try_default() {
            Ok(pair) => (Some(pair.0), Some(pair.1)),
            Err(_) => (None, None),
        };
        Self {
            _stream: stream,
            handle,
            playing: HashSet::new(),
            asset_roots,
        }
    }

    pub fn play_clip(&mut self, clip_path: &str, volume: f32, looping: bool) {
        let Some(handle) = &self.handle else {
            return;
        };
        let Some(full_path) = self.resolve_path(clip_path) else {
            return;
        };
        let Ok(file) = File::open(&full_path) else {
            return;
        };
        let reader = BufReader::new(file);
        let Ok(source) = rodio::Decoder::new(reader) else {
            return;
        };
        let Ok(sink) = rodio::Sink::try_new(handle) else {
            return;
        };
        sink.set_volume(volume.clamp(0.0, 1.0));
        if looping {
            use rodio::Source;
            sink.append(source.repeat_infinite());
        } else {
            sink.append(source);
        }
        sink.detach();
        self.playing.insert(clip_path.to_string());
    }

    /// Plays the clip only if it hasn't been played yet in this session.
    pub fn play_clip_once(&mut self, clip_path: &str, volume: f32, looping: bool) {
        if !self.playing.contains(clip_path) {
            self.play_clip(clip_path, volume, looping);
        }
    }

    pub fn set_asset_roots(&mut self, roots: Vec<PathBuf>) {
        self.asset_roots = roots;
    }

    fn resolve_path(&self, clip_path: &str) -> Option<PathBuf> {
        for root in &self.asset_roots {
            let candidate = root.join(clip_path);
            if candidate.exists() {
                return Some(candidate);
            }
        }
        // Also try stripping leading "audio/" etc.
        let stripped = Path::new(clip_path).file_name()?;
        for root in &self.asset_roots {
            let candidate = root.join(stripped);
            if candidate.exists() {
                return Some(candidate);
            }
        }
        None
    }
}
