$ErrorActionPreference = 'Stop'

Get-Process 'Sucrose*' | % {
	Write-Output ('Closing: {0}' -f $_.ProcessName)
	Stop-Process -InputObject $_ -Force
}

$packageArgs = @{
	packageName    = 'SucroseWallpaperEngine'
	checksumType   = 'sha256'
	fileType       = 'exe'
	silentArgs     = '/s'
	validExitCodes = @(0)
}

if ($env:PROCESSOR_ARCHITECTURE -eq 'ARM64') {
	$packageArgs['checksum'] = '4431DDAA9266EAC9738CCD053338998C472B6853D65C949CF6263485E6D1AB6C'
	$packageArgs['url'] = 'https://github.com/Taiizor/Sucrose/releases/download/v24.10.1.0/Sucrose_Bundle_.NET_Framework_4.8_ARM64_24.10.1.0.exe'
} else {
	if ([Environment]::Is64BitOperatingSystem) {
		if ([System.Environment]::Is64BitProcess) {
			$packageArgs['checksum'] = '955AE920E988E374710ADB24966C37A936AB600F3BB41896FAA195E68DCAABB7'
			$packageArgs['url'] = 'https://github.com/Taiizor/Sucrose/releases/download/v24.10.1.0/Sucrose_Bundle_.NET_Framework_4.8_x64_24.10.1.0.exe'
		} else {
			$packageArgs['checksum'] = 'EB03A2C268A69F48D029BA3B97E7B5E6F5BA5D70D680D97B910A448D69BEFA75'
			$packageArgs['url'] = 'https://github.com/Taiizor/Sucrose/releases/download/v24.10.1.0/Sucrose_Bundle_.NET_Framework_4.8_x86_24.10.1.0.exe'
		}
	} else {
		$packageArgs['checksum'] = 'EB03A2C268A69F48D029BA3B97E7B5E6F5BA5D70D680D97B910A448D69BEFA75'
		$packageArgs['url'] = 'https://github.com/Taiizor/Sucrose/releases/download/v24.10.1.0/Sucrose_Bundle_.NET_Framework_4.8_x86_24.10.1.0.exe'
	}
}

Install-ChocolateyPackage @packageArgs