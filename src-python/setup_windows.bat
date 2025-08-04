@echo off
setlocal enabledelayedexpansion

@rem environment isolation
set PYTHONNOUSERSITE=1
set PYTHONPATH=
set PYTHONHOME=

cd /D "%~dp0"

set PATH=%PATH%;%SystemRoot%\system32
echo PWD = %CD%
echo "%CD%"| findstr /C:" " >nul && echo This script relies on Miniconda which can not be silently installed under a path with spaces. && goto end

@rem Check for special characters in installation path
set "SPCHARMESSAGE="WARNING: Special characters were detected in the installation path!" "         This can cause the installation to fail!""
echo "%CD%"| findstr /R /C:"[!#\$%&()\*+,;<=>?@\[\]\^`{|}~]" >nul && (
	call :PrintBigMessage %SPCHARMESSAGE%
)
set SPCHARMESSAGE=

@rem fix failed install when installing to a separate drive
set TMP=%cd%\installation
set TEMP=%cd%\installation

@rem deactivate existing conda envs as needed to avoid conflicts
(call conda deactivate && call conda deactivate && call conda deactivate) 2>nul

@rem config
set INSTALL_DIR=%cd%\installation
set CONDA_ROOT_PREFIX=%cd%\installation\conda
set INSTALL_ENV_DIR=%cd%\installation\env
set MINICONDA_DOWNLOAD_URL=https://repo.anaconda.com/miniconda/Miniconda3-py312_25.5.1-1-Windows-x86_64.exe
set MINICONDA_CHECKSUM=0905638dcd7bb214261fe0ac6136d4dbbb9c4266fa3cf4c5bf4ffce834ae5357
set conda_exists=F

@rem figure out whether git and conda needs to be installed
call "%CONDA_ROOT_PREFIX%\_conda.exe" --version >nul 2>&1
if "%ERRORLEVEL%" EQU "0" set conda_exists=T

@rem (if necessary) install git and conda into a contained environment
@rem download conda
if "%conda_exists%" == "F" (
	echo Downloading Miniconda from %MINICONDA_DOWNLOAD_URL% to %INSTALL_DIR%\miniconda_installer.exe

	mkdir "%INSTALL_DIR%"
	call curl -Lk "%MINICONDA_DOWNLOAD_URL%" > "%INSTALL_DIR%\miniconda_installer.exe" || ( echo. && echo Miniconda failed to download. && goto end )

	@rem Try CertUtil first
	for /f %%a in ('CertUtil -hashfile "%INSTALL_DIR%\miniconda_installer.exe" SHA256 ^| find /i /v " " ^| find /i "%MINICONDA_CHECKSUM%"') do (
		set "output=%%a"
	)

	@rem If CertUtil fails, try PowerShell
	if not defined output (
		for /f %%a in ('powershell -Command "if((Get-FileHash \"%INSTALL_DIR%\miniconda_installer.exe\" -Algorithm SHA256).Hash -eq ''%MINICONDA_CHECKSUM%''){echo true}"') do (
			set "output=%%a"
		)
	)

	if not defined output (
		echo The checksum verification for miniconda_installer.exe has failed.
		del "%INSTALL_DIR%\miniconda_installer.exe"
		goto end
	) else (
		echo The checksum verification for miniconda_installer.exe has passed successfully.
	)

	echo Installing Miniconda to %CONDA_ROOT_PREFIX%
	start /wait "" "%INSTALL_DIR%\miniconda_installer.exe" /InstallationType=JustMe /NoShortcuts=1 /AddToPath=0 /RegisterPython=0 /NoRegistry=1 /S /D=%CONDA_ROOT_PREFIX%

	@rem test the conda binary
	echo Miniconda version:
	call "%CONDA_ROOT_PREFIX%\_conda.exe" --version || ( echo. && echo Miniconda not found. && goto end )

	@rem delete the Miniconda installer
	del "%INSTALL_DIR%\miniconda_installer.exe"
)

@rem create the installer env
if not exist "%INSTALL_ENV_DIR%" (
	echo Packages to install: %PACKAGES_TO_INSTALL%
	call "%CONDA_ROOT_PREFIX%\_conda.exe" create --no-shortcuts -y -k --prefix "%INSTALL_ENV_DIR%" python=3.12 || ( echo. && echo Conda environment creation failed. && goto end )
)

@rem check if conda environment was actually created
if not exist "%INSTALL_ENV_DIR%\python.exe" ( echo. && echo Conda environment is empty. && goto end )

@REM set "CUDA_PATH=%INSTALL_ENV_DIR%"
@REM set "CUDA_HOME=%CUDA_PATH%"
set "CI=1"

@rem activate installer env
call "%CONDA_ROOT_PREFIX%\condabin\conda.bat" activate "%INSTALL_ENV_DIR%" || ( echo. && echo Miniconda hook not found. && goto end )

@rem setup installer env
call python setup.py %*

@rem below are functions for the script   next line skips these during normal execution
goto end

:PrintBigMessage
echo. && echo.
echo *******************************************************************
for %%M in (%*) do echo * %%~M
echo *******************************************************************
echo. && echo.
exit /b

:end
pause