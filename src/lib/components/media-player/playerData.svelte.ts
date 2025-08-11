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
}

export type AudioInfo = {
    name: string; // Name of the audio file
    artist?: string; // Artist or short description of the audio file
    thumbnail?: string | ImageBase64String; // Path to the thumbnail image or Base64 encoded string
    path: string; // Path to the audio file
    duration: number; // Duration of the audio file in seconds
    size: number; // Size of the audio file in bytes
    sampleRate: number; // Sample rate of the audio file
    channels: number; // Number of audio channels

    chapters?: AudioChapterInfo[];
}

export type MediaPlayerDataType = {
    audioInfo: AudioInfo | null; // Information about the currently loaded audio file, null if no media loaded

    isPlaying: boolean;
    progress: number | null; // Current playback progress, [0-100], null == no media loaded
    duration: number | null; // Total duration of the media in seconds, same as audioInfo.duration for convenience, or null if not applicable/media not loaded
    volume: number; // Current volume level, [0-100]
    muted: boolean; // Mute state of the media player
    loop: "none" | "all" | "one";

    skipDuration: number; // Duration to skip forward/backward in seconds

    alwaysShowPlayer: boolean; // Whether to always show the player UI, even when no media is loaded
};

const defaultAudioInfo = {
    name: "Dreamy Night",
    artist: "comfi beats",
    thumbnail: undefined,
    path: "",
    duration: 0,
    size: 0,
    sampleRate: 0,
    channels: 0,

    chapters: [
        {
            timestamp: 0,
            title: "Introduction",
        },
        {
            timestamp: 40,
            title: "Verse 1: Left",
        },
        {
            timestamp: 60,
            title: "really long title just to test if this works",
        },
    ]
}

export const MediaPlayerData: MediaPlayerDataType = $state({
    // audioInfo: null,
    audioInfo: defaultAudioInfo || null,

    isPlaying: false,
    progress: null,
    duration: defaultAudioInfo.duration || 100,
    volume: 80,
    muted: false,
    loop: "none",

    skipDuration: 5, // Duration to skip forward/backward in seconds

    alwaysShowPlayer: false, // Whether to always show the player UI, even when no media is loaded
});