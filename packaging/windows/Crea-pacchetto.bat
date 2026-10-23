@echo off
rem Crea il pacchetto DocPlane per Windows: scarica (o aggiorna) WordBook.it, SheetCalc.it e
rem Flickdeck, compila i tre programmi e DocPlane, li mette insieme in dist\DocPlane e crea
rem dist\DocPlane.zip. Va lanciato con Rust e il compilatore C gia' nel PATH.
rem Le cartelle dei programmi stanno accanto a quella di DocPlane.
setlocal
cd /d "%~dp0\..\.."
set "ROOT=%CD%"
for %%I in ("%ROOT%\..") do set "BASE=%%~fI"
set "DIST=%ROOT%\dist\DocPlane"
if exist "%DIST%" rmdir /s /q "%DIST%"
mkdir "%DIST%"

echo === DocPlane ===
cargo build --release -p docplane
if errorlevel 1 goto :fail
copy /Y target\release\docplane.exe "%DIST%\DocPlane.exe" >nul

call :build Wordbook.it wordbook.it wordcraft wordbook.exe WordBook.it.exe
if errorlevel 1 goto :fail
call :build SheetCalc.it-src SheetCalc.it gridcraft gridcraft.exe SheetCalc.it.exe
if errorlevel 1 goto :fail
call :build Flickdeck Flickdeck flickdeck flickdeck.exe Flickdeck.exe
if errorlevel 1 goto :fail

rem Modalita' portatile: ogni programma tiene le impostazioni in una cartella accanto a se'.
mkdir "%DIST%\docplane-data"
mkdir "%DIST%\wordbook-data"
mkdir "%DIST%\flickdeck-data"
type nul > "%DIST%\portable.txt"

powershell -NoProfile -Command "Compress-Archive -Path '%DIST%\*' -DestinationPath '%ROOT%\dist\DocPlane.zip' -Force"
echo.
echo Fatto: %DIST%\DocPlane.exe
echo Zip:   %ROOT%\dist\DocPlane.zip
exit /b 0

:fail
echo.
echo ERRORE: il pacchetto non e' stato creato.
exit /b 1

:build
rem %1 cartella  %2 repository GitHub  %3 pacchetto cargo  %4 file compilato  %5 nome nel pacchetto
echo === %~5 ===
if exist "%BASE%\%~1" (
  git -C "%BASE%\%~1" pull
) else (
  git clone https://github.com/lorenzodapolito-oss/%~2 "%BASE%\%~1"
)
if errorlevel 1 exit /b 1
pushd "%BASE%\%~1"
cargo build --release -p %~3
if errorlevel 1 (
  popd
  exit /b 1
)
copy /Y "target\release\%~4" "%DIST%\%~5" >nul
popd
exit /b 0
