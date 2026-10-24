@echo off
rem Crea il pacchetto DocPlane.it per Windows: scarica (o aggiorna) WordBook.it, SheetCalc.it e
rem Flickdeck.it, compila i tre programmi e DocPlane.it, li mette insieme in dist\DocPlane.it,
rem scarica i dizionari (italiano e inglese) e crea dist\DocPlane.it.zip. Va lanciato con Rust e il compilatore C gia' nel PATH.
rem Le cartelle dei programmi stanno dentro quella di DocPlane.
setlocal
cd /d "%~dp0\..\.."
set "ROOT=%CD%"
set "BASE=%ROOT%"
set "DIST=%ROOT%\dist\DocPlane.it"
if exist "%DIST%" rmdir /s /q "%DIST%"
mkdir "%DIST%"

echo === DocPlane.it ===
cargo build --release -p docplane
if errorlevel 1 goto :fail
copy /Y target\release\docplane.exe "%DIST%\DocPlane.it.exe" >nul

call :build Wordbook.it wordbook.it wordcraft wordbook.exe WordBook.it.exe
if errorlevel 1 goto :fail
call :build SheetCalc.it-src SheetCalc.it gridcraft gridcraft.exe SheetCalc.it.exe
if errorlevel 1 goto :fail
call :build Flickdeck Flickdeck flickdeck flickdeck.exe Flickdeck.it.exe
if errorlevel 1 goto :fail

rem Le cartelle vuote non entrano nello zip: ognuna ha un file LEGGIMI.txt.
rem Modalita' portatile: ogni programma tiene le impostazioni in una cartella accanto a se'.
mkdir "%DIST%\docplane-data"
echo Impostazioni di questo programma (modalita portatile).> "%DIST%\docplane-data\LEGGIMI.txt"
mkdir "%DIST%\sheetcalc-data"
echo Impostazioni di questo programma (modalita portatile).> "%DIST%\sheetcalc-data\LEGGIMI.txt"
mkdir "%DIST%\wordbook-data"
echo Impostazioni di questo programma (modalita portatile).> "%DIST%\wordbook-data\LEGGIMI.txt"
mkdir "%DIST%\flickdeck-data"
echo Impostazioni di questo programma (modalita portatile).> "%DIST%\flickdeck-data\LEGGIMI.txt"
type nul > "%DIST%\portable.txt"

echo === Dizionari ===
rem DocPlane.it e' un programma a finestre: start /wait aspetta che finisca.
start /wait "" "%DIST%\DocPlane.it.exe" --scarica-dizionari "%DIST%\dictionaries"
if errorlevel 1 echo ATTENZIONE: i dizionari non sono stati scaricati. Si possono scaricare dopo da DocPlane.it.

powershell -NoProfile -Command "Compress-Archive -Path '%DIST%\*' -DestinationPath '%ROOT%\dist\DocPlane.it.zip' -Force"
echo.
echo Fatto: %DIST%\DocPlane.it.exe
echo Zip:   %ROOT%\dist\DocPlane.it.zip
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
