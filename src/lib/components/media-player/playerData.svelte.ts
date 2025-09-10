const imageBase64Regex = /^data:image\/(png|jpe?g|gif|webp|svg\+xml|bmp|tiff?);base64,[A-Za-z0-9+/]+=*$/;
export type ImageBase64String = string & { readonly __brand: 'ImageBase64String' };

// Type guard function to validate and narrow to ImageBase64String
export function isImageBase64String(value: string): value is ImageBase64String {
    return imageBase64Regex.test(value);
}

// Helper function to create ImageBase64String (throws if invalid)
export function asImageBase64String(value: string): ImageBase64String {
    if (!isImageBase64String(value)) {
        throw new Error('Invalid base64 image data URL format');
    }
    return value as ImageBase64String;
}

// Helper function to safely create ImageBase64String (returns null if invalid)
export function tryCastImageBase64String(value: string): ImageBase64String | null {
    return isImageBase64String(value) ? value as ImageBase64String : null;
}

export type AudioChapterInfo = {
    timestamp: number; // Timestamp of the chapter in seconds
    title: string; // Title of the chapter
    description?: string; // Extra information about the chapter
    startsample?: number; // The start sample of the chapter, relative to the start of the audio file
}

export type AudioInfo = {
    name: string; // Name of the audio file
    artist?: string; // Artist or short description of the audio file
    contributors?: string; // Additional contributing artist or information
    thumbnail?: string | ImageBase64String; // Path to the thumbnail image or Base64 encoded string
    path: string; // Path to the audio file
    duration: number; // Duration of the audio file in seconds
    size: number; // Size of the audio file in bytes
    sampleRate: number; // Sample rate of the audio file
    channels: number; // Number of audio channels
    bitDepth: number; // Bit depth of the audio file

    chapters?: AudioChapterInfo[];
}

export type MediaPlayerDataType = {
    audioInfo: AudioInfo | null; // Information about the currently loaded audio file, null if no media loaded

    isPlaying: boolean;
    playbackCompleted: boolean;
    progress: number | null; // Current playback progress, [0-100], null == no media loaded
    duration: number | null; // Total duration of the media in seconds, same as audioInfo.duration for convenience, or null if not applicable/media not loaded
    volume: number; // Current volume level, [0-100]
    muted: boolean; // Mute state of the media player
    loop: "none" | "all" | "one";
    flipLRStereo: boolean; // Whether to flip left/right stereo channels, only apply to stereo audio

    pauseAutomaticWsProgressUpdate: boolean; // When this is true, progress received from WS messages are not applied to GUI

    skipDuration: number; // Duration to skip forward/backward in seconds

    alwaysShowPlayer: boolean; // Whether to always show the player UI, even when no media is loaded
};

export const MediaPlayerData: MediaPlayerDataType = $state({
    audioInfo: null,

    isPlaying: false,
    playbackCompleted: false,
    progress: null,
    duration: 420,
    volume: 50,
    muted: false,
    loop: "none",
    flipLRStereo: false,

    pauseAutomaticWsProgressUpdate: false,

    skipDuration: 5, // Duration to skip forward/backward in seconds

    alwaysShowPlayer: false, // Whether to always show the player UI, even when no media is loaded
});