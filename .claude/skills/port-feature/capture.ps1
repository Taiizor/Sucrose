param(
    [Parameter(Mandatory = $true)][string]$Process,
    [Parameter(Mandatory = $true)][string]$Out
)

# Captures a window with PrintWindow (works when the window is behind others)
# and prints its size and memory use. Does not activate the window.
Add-Type -AssemblyName System.Drawing
Add-Type @"
using System;
using System.Runtime.InteropServices;
public class SucroseCapture {
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr h, IntPtr hdc, uint flags);
  public struct RECT { public int L, T, R, B; }
}
"@
[SucroseCapture]::SetProcessDPIAware() | Out-Null

$p = Get-Process -Name $Process -ErrorAction SilentlyContinue | Where-Object { $_.MainWindowHandle -ne 0 } | Select-Object -First 1
if (-not $p) { Write-Output "NO_WINDOW"; exit 1 }

$h = $p.MainWindowHandle
$r = New-Object SucroseCapture+RECT
[SucroseCapture]::GetWindowRect($h, [ref]$r) | Out-Null
$w = $r.R - $r.L
$hh = $r.B - $r.T

$bmp = New-Object System.Drawing.Bitmap $w, $hh
$g = [System.Drawing.Graphics]::FromImage($bmp)
$hdc = $g.GetHdc()
[SucroseCapture]::PrintWindow($h, $hdc, 2) | Out-Null  # PW_RENDERFULLCONTENT
$g.ReleaseHdc($hdc)
$bmp.Save($Out, [System.Drawing.Imaging.ImageFormat]::Png)

$p.Refresh()
"{0}x{1} workingset={2:N1}MB private={3:N1}MB" -f $w, $hh, ($p.WorkingSet64 / 1MB), ($p.PrivateMemorySize64 / 1MB)
