

export type AudioInfo = {
    name: string; // Name of the audio file
    path: string; // Path to the audio file
    duration: number; // Duration of the audio file in seconds
    size: number; // Size of the audio file in bytes
    sampleRate: number; // Sample rate of the audio file
    channels: number; // Number of audio channels
}

export type MediaPlayerDataType = {
    audioInfo: AudioInfo | null; // Information about the currently loaded audio file, null if no media loaded
    progress: number | null; // Current playback progress, [0-100], null == no media loaded
    duration: number | null; // Total duration of the media in seconds, same as audioInfo.duration for convenience, or null if not applicable/media not loaded
    volume: number; // Current volume level, [0-100]
    muted: boolean; // Mute state of the media player

    skipDuration: number; // Duration to skip forward/backward in seconds

    alwaysShowPlayer: boolean; // Whether to always show the player UI, even when no media is loaded
};

export const MediaPlayerData: MediaPlayerDataType = $state({
    audioInfo: null, // Initially no media loaded
    progress: null,
    duration: 100,
    volume: 80,
    muted: false,

    skipDuration: 5, // Duration to skip forward/backward in seconds

    alwaysShowPlayer: true, // Whether to always show the player UI, even when no media is loaded
} as MediaPlayerDataType);