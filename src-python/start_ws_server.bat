@echo off

title NI-DAQmx Audio Player WS Server

cd /D "%~dp0"

set PATH=%PATH%;%SystemRoot%\system32

echo "%CD%"| findstr /C:" " >nul && echo This script relies on Miniconda which can not be silently installed under a path with spaces. && goto end

@rem fix failed install when installing to a separate drive
set TMP=%cd%\installation
set TEMP=%cd%\installation

@rem deactivate existing conda envs as needed to avoid conflicts
(call conda deactivate && call conda deactivate && call conda deactivate) 2>nul

echo Starting the Python WebSocket server...
echo This window will hide automatically in just a moment ^:^>
echo:

@rem config
set CONDA_ROOT_PREFIX=%cd%\installation\conda
set INSTALL_ENV_DIR=%cd%\installation\env

@rem environment isolation
set PYTHONNOUSERSITE=1
set PYTHONPATH=
set PYTHONHOME=
@REM set "CUDA_PATH=%INSTALL_ENV_DIR%"
@REM set "CUDA_HOME=%CUDA_PATH%"

@rem activate installer env
echo [1/2] Activating conda environment "%INSTALL_ENV_DIR%"...
call "%CONDA_ROOT_PREFIX%\condabin\conda.bat" activate "%INSTALL_ENV_DIR%" || ( echo. && echo Miniconda hook not found. && goto end )

@REM start the Python WebSocket server
echo [2/2] Booting up the WebSocket server...
echo:
call python src\main.py %* || ( echo. && echo Failed to start the Python WebSocket server. && goto end )

echo WebSocket server started successfully

@REM enter commands
@REM cmd /k "%*"

:end