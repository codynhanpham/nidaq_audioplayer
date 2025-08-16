import json
import asyncio
from typing import Dict, Any
from . import load_audio
from ..utils import create_success_response, create_error_response, broadcast_message


async def handle_play(websocket, data: Any) -> Dict[str, Any]:
    """Handle play task - starts audio playback with progress updates."""
    
    try:
        if load_audio.nidaq_player is None:
            return create_error_response("No audio player initialized. Load audio first.")
        
        if not load_audio.nidaq_player._audio_loaded:
            return create_error_response("No audio file loaded. Load audio first.")
        
        # Start playback
        load_audio.nidaq_player.play()
        
        # Start progress monitoring task in background
        asyncio.create_task(monitor_playback_progress(websocket))
        
        # Return initial success response
        response_data = {
            "message": "Playback started",
            "status": load_audio.nidaq_player.get_status()
        }
        
        return create_success_response(response_data)
        
    except Exception as e:
        return create_error_response(f"Play task failed: {str(e)}")


async def monitor_playback_progress(websocket):
    """Background task to monitor playback progress and send updates."""
    
    try:
        while load_audio.nidaq_player and load_audio.nidaq_player._playing:
            status = load_audio.nidaq_player.get_status()
            
            # Send progress update
            progress_response = {
                "id": "progress_update",
                "timestamp": int(asyncio.get_event_loop().time() * 1000),
                "status": "progress",
                "data": {
                    "current_time": status.get('current_time', 0),
                    "duration": status.get('duration', 0),
                    "progress_percent": (status.get('current_time', 0) / status.get('duration', 1)) * 100 if status.get('duration', 0) > 0 else 0,
                    "playing": status.get('playing', False),
                    "audio_completed": status.get('audio_completed', False),
                    "samples_generated": status.get('samples_generated', 0),
                    "voltage_scale": getattr(load_audio.nidaq_player, 'voltage_scale', 0.1),
                    "raw_status": status
                },
                "completed": False
            }
            
            try:
                await websocket.send(json.dumps(progress_response))
            except Exception as e:
                print(f"Failed to send progress update: {e}")
                break
            
            # Check if playback completed
            if status.get('audio_completed', False):
                # Send completion notification
                completion_response = {
                    "id": "playback_completed",
                    "timestamp": int(asyncio.get_event_loop().time() * 1000),
                    "status": "completed",
                    "data": {
                        "message": "Playback completed",
                        "final_status": status
                    },
                    "completed": True
                }
                
                load_audio.nidaq_player._clear_tasks()
                try:
                    await websocket.send(json.dumps(completion_response))
                except Exception as e:
                    print(f"Failed to send completion notification: {e}")
                break

            await asyncio.sleep(0.075)

        # Check if playback completed
    # if status.get('audio_completed', False):
        # Send completion notification
        completion_response = {
            "id": "playback_completed",
            "timestamp": int(asyncio.get_event_loop().time() * 1000),
            "status": "completed",
            "data": {
                "message": "Playback completed",
                "final_status": load_audio.nidaq_player.get_status()
            },
            "completed": True
        }
        
        load_audio.nidaq_player._clear_tasks()
        try:
            await websocket.send(json.dumps(completion_response))
        except Exception as e:
            print(f"Failed to send completion notification: {e}")
    
    except Exception as e:
        print(f"Error during progress monitoring: {e}")
        
        # Send error notification
        error_response = {
            "id": "progress_error",
            "timestamp": int(asyncio.get_event_loop().time() * 1000),
            "status": "error",
            "data": {
                "error": f"Progress monitoring failed: {str(e)}"
            },
            "completed": True
        }
        
        try:
            await websocket.send(json.dumps(error_response))
        except:
            pass