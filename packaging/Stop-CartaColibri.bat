@echo off
taskkill /im cartacolibri.exe /f >nul 2>&1 && echo CartaColibri stopped. || echo CartaColibri was not running.
timeout /t 2 >nul
