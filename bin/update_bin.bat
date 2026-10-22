@echo off
chcp 65001 >nul
title 核心依赖组件巡检与更新工具

echo ========================================================
echo        E:\.Cli\bin 专用依赖引擎版本巡检与更新
echo ========================================================
echo.

cd /d "%~dp0"

:: 1. 检查 Aria2
echo [1/3] 检查 Aria2 (大文件/表格/多线程下载引擎):
if exist "aria2c.exe" (
    aria2c.exe -v | findstr /C:"aria2 version"
    echo [状态] 正常就位
) else (
    echo [缺失] 未找到 aria2c.exe，请补充放入当前目录！
)
echo --------------------------------------------------------

:: 2. 检查 Monolith
echo [2/3] 检查 Monolith (单文件网页打包引擎):
if exist "monolith.exe" (
    monolith.exe --version
    echo [状态] 正常就位
) else (
    echo [缺失] 未找到 monolith.exe，请补充放入当前目录！
)
echo --------------------------------------------------------

:: 3. 检查并在线更新 yt-dlp
echo [3/3] 检查与更新 yt-dlp (流媒体音视频解析引擎):
if exist "yt-dlp.exe" (
    yt-dlp.exe --version
    echo 正在检查并拉取最新规则库版本...
    yt-dlp.exe -U
) else (
    echo [缺失] 未找到 yt-dlp.exe，请补充放入当前目录！
)
echo --------------------------------------------------------

echo.
echo 巡检完毕。
pause