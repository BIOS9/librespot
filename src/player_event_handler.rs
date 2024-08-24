use log::{debug, error, warn};

use std::thread;

use librespot::{
    metadata::audio::UniqueFields,
    playback::player::{PlayerEvent, PlayerEventChannel, SinkStatus},
};

use serde_json::{json, Value};

pub struct EventHandler {
    thread_handle: Option<thread::JoinHandle<()>>,
}

impl EventHandler {
    pub fn new(mut player_events: PlayerEventChannel) -> Self {
        let thread_handle = Some(thread::spawn(move || loop {
            match player_events.blocking_recv() {
                None => break,
                Some(event) => {
                    let mut event_name: &str = "unknown";
                    let mut json_obj: Option<Value> = None;                    

                    match event.clone() {
                        PlayerEvent::PlayRequestIdChanged { play_request_id } => {
                            event_name = "playRequestIdChanged";
                            json_obj = Some(json!({
                                "playRequestId": play_request_id,
                            }));
                        }
                        PlayerEvent::TrackChanged { audio_item } => {
                            event_name = "trackChanged";
                            match audio_item.track_id.to_base62() {
                                Err(e) => {
                                    warn!("PlayerEvent::TrackChanged: Invalid track id: {}", e)
                                }
                                Ok(id) => {
                                    json_obj = Some(json!({
                                        "trackId": id,
                                        "uri": audio_item.uri,
                                        "name": audio_item.name,
                                        "covers": audio_item.covers
                                            .into_iter()
                                            .map(|c| c.url)
                                            .collect::<Vec<String>>(),
                                        "language": audio_item.language,
                                        "durationMs": audio_item.duration_ms,
                                        "isExplicit": audio_item.is_explicit,
                                        "extraFields": match audio_item.unique_fields {
                                            UniqueFields::Track {
                                                artists,
                                                album,
                                                album_artists,
                                                popularity,
                                                number,
                                                disc_number,
                                            } => Some(json!({
                                                "itemType": "Track",
                                                "albumArtists": album_artists,
                                                "album": album,
                                                "popularity": popularity,
                                                "number": number,
                                                "discNumber": disc_number,
                                                "artists": artists
                                                    .0
                                                    .into_iter()
                                                    .map(|a| a.name)
                                                    .collect::<Vec<String>>(),
                                            })),
                                            UniqueFields::Episode {
                                                description,
                                                publish_time,
                                                show_name,
                                            } => Some(json!({
                                                "itemType": "Episode",
                                                "description": description,
                                                "publishTime": publish_time.unix_timestamp(),
                                                "showName": show_name,
                                            })),
                                        },
                                    }));
                                }
                            }
                        }
                        PlayerEvent::Stopped { track_id, .. } => match track_id.to_base62() {
                            Err(e) => warn!("PlayerEvent::Stopped: Invalid track id: {}", e),
                            Ok(id) => {
                                event_name = "stopped";
                                json_obj = Some(json!({
                                    "trackId": id,
                                }));
                            }
                        },
                        PlayerEvent::Playing {
                            track_id,
                            position_ms,
                            ..
                        } => match track_id.to_base62() {
                            Err(e) => warn!("PlayerEvent::Playing: Invalid track id: {}", e),
                            Ok(id) => {
                                event_name = "playing";
                                json_obj = Some(json!({
                                    "trackId": id,
                                    "positionMs": position_ms,
                                }));
                            }
                        },
                        PlayerEvent::Paused {
                            track_id,
                            position_ms,
                            ..
                        } => match track_id.to_base62() {
                            Err(e) => warn!("PlayerEvent::Paused: Invalid track id: {}", e),
                            Ok(id) => {
                                event_name = "paused";
                                json_obj = Some(json!({
                                    "trackId": id,
                                    "positionMs": position_ms,
                                }));
                            }
                        },
                        PlayerEvent::Loading { track_id, .. } => match track_id.to_base62() {
                            Err(e) => warn!("PlayerEvent::Loading: Invalid track id: {}", e),
                            Ok(id) => {
                                event_name = "loading";
                                json_obj = Some(json!({
                                    "trackId": id,
                                }));
                            }
                        },
                        PlayerEvent::Preloading { track_id, .. } => match track_id.to_base62() {
                            Err(e) => warn!("PlayerEvent::Preloading: Invalid track id: {}", e),
                            Ok(id) => {
                                event_name = "preloading";
                                json_obj = Some(json!({
                                    "trackId": id,
                                }));
                            }
                        },
                        PlayerEvent::TimeToPreloadNextTrack { track_id, .. } => {
                            match track_id.to_base62() {
                                Err(e) => warn!(
                                    "PlayerEvent::TimeToPreloadNextTrack: Invalid track id: {}",
                                    e
                                ),
                                Ok(id) => {
                                    event_name = "timeToPreloadNextTrack";
                                    json_obj = Some(json!({
                                        "trackId": id,
                                    }));
                                }
                            }
                        }
                        PlayerEvent::EndOfTrack { track_id, .. } => match track_id.to_base62() {
                            Err(e) => warn!("PlayerEvent::EndOfTrack: Invalid track id: {}", e),
                            Ok(id) => {
                                event_name = "endOfTrack";
                                json_obj = Some(json!({
                                    "trackId": id,
                                }));
                            }
                        },
                        PlayerEvent::Unavailable { track_id, .. } => match track_id.to_base62() {
                            Err(e) => warn!("PlayerEvent::Unavailable: Invalid track id: {}", e),
                            Ok(id) => {
                                event_name = "unavailable";
                                json_obj = Some(json!({
                                    "trackId": id,
                                }));
                            }
                        },
                        PlayerEvent::VolumeChanged { volume } => {
                            event_name = "volumeChanged";
                            json_obj = Some(json!({
                                "volume": volume,
                            }));
                        }
                        PlayerEvent::Seeked {
                            track_id,
                            position_ms,
                            ..
                        } => match track_id.to_base62() {
                            Err(e) => warn!("PlayerEvent::Seeked: Invalid track id: {}", e),
                            Ok(id) => {
                                event_name = "seeked";
                                json_obj = Some(json!({
                                    "trackId": id,
                                    "positionMs": position_ms,
                                }));
                            }
                        },
                        PlayerEvent::PositionCorrection {
                            track_id,
                            position_ms,
                            ..
                        } => match track_id.to_base62() {
                            Err(e) => {
                                warn!("PlayerEvent::PositionCorrection: Invalid track id: {}", e)
                            }
                            Ok(id) => {
                                event_name = "positionCorrection";
                                json_obj = Some(json!({
                                    "trackId": id,
                                    "positionMs": position_ms,
                                }));
                            }
                        },
                        PlayerEvent::SessionConnected {
                            connection_id,
                            user_name,
                        } => {
                            event_name = "sessionConnected";
                            json_obj = Some(json!({
                                "connectionId": connection_id,
                                "userName": user_name,
                            }));
                        }
                        PlayerEvent::SessionDisconnected {
                            connection_id,
                            user_name,
                        } => {
                            event_name = "sessionDisconnected";
                            json_obj = Some(json!({
                                "connectionId": connection_id,
                                "userName": user_name,
                            }));
                        }
                        PlayerEvent::SessionClientChanged {
                            client_id,
                            client_name,
                            client_brand_name,
                            client_model_name,
                        } => {
                            event_name = "sessionClientChanged";
                            json_obj = Some(json!({
                                "clientId": client_id,
                                "clientName": client_name,
                                "clientBrandName": client_brand_name,
                                "cleintModelName": client_model_name,
                            }));
                        }
                        PlayerEvent::ShuffleChanged { shuffle } => {
                            event_name = "shuffleChanged";
                            json_obj = Some(json!({
                                "shuffle": shuffle,
                            }));
                        }
                        PlayerEvent::RepeatChanged { repeat } => {
                            event_name = "repeatChanged";
                            json_obj = Some(json!({
                                "repeat": repeat,
                            }));
                        }
                        PlayerEvent::AutoPlayChanged { auto_play } => {
                            event_name = "autoPlayChanged";
                            json_obj = Some(json!({
                                "autoPlay": auto_play,
                            }));
                        }

                        PlayerEvent::FilterExplicitContentChanged { filter } => {
                            event_name = "filterExplicitContentChanged";
                            json_obj = Some(json!({
                                "filter": filter,
                            }));
                        }
                    }

                    if let Some(json_obj) = json_obj {
                        match serde_json::to_string(&json_obj) {
                            Ok(s) => {
                                eprintln!("raise_event {} {}", event_name, s);
                            },
                            Err(e) => {
                                warn!("Failed to serialize PlayerEvent: {}", e);
                                continue;
                            }
                        };
                    }
                }
            }
        }));

        Self { thread_handle }
    }
}

impl Drop for EventHandler {
    fn drop(&mut self) {
        debug!("Shutting down EventHandler thread ...");
        if let Some(handle) = self.thread_handle.take() {
            if let Err(e) = handle.join() {
                error!("EventHandler thread Error: {:?}", e);
            }
        }
    }
}

pub fn handle_sink_events(sink_status: SinkStatus) {
    eprintln!("raise_event sink {}", json!({
        "sinkStatus": match sink_status {
            SinkStatus::Running => "running",
            SinkStatus::TemporarilyClosed => "temporarily_closed",
            SinkStatus::Closed => "closed",
        }
    }));
}
