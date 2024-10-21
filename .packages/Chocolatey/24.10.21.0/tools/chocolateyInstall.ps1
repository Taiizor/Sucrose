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
	$packageArgs['checksum'] = 'D0B60DC937F36E596B4CF04EB8992720E900F5A2EA12CF171A443213445BED76'
	$packageArgs['url'] = 'https://github.com/Taiizor/Sucrose/releases/download/v24.10.21.0/Sucrose_Bundle_.NET_Framework_4.8_ARM64_24.10.21.0.exe'
} else {
	if ([Environment]::Is64BitOperatingSystem) {
		if ([System.Environment]::Is64BitProcess) {
			$packageArgs['checksum'] = 'EB0BED54D4F4122ECCC55D5815D6C98B54D413A400E534D47D85704B4C4F59B0'
			$packageArgs['url'] = 'https://github.com/Taiizor/Sucrose/releases/download/v24.10.21.0/Sucrose_Bundle_.NET_Framework_4.8_x64_24.10.21.0.exe'
		} else {
			$packageArgs['checksum'] = 'EA27984F7F447473BB541ACB7DC72D79EC356E65DDCA19153152C051BA489A10'
			$packageArgs['url'] = 'https://github.com/Taiizor/Sucrose/releases/download/v24.10.21.0/Sucrose_Bundle_.NET_Framework_4.8_x86_24.10.21.0.exe'
		}
	} else {
		$packageArgs['checksum'] = 'EA27984F7F447473BB541ACB7DC72D79EC356E65DDCA19153152C051BA489A10'
		$packageArgs['url'] = 'https://github.com/Taiizor/Sucrose/releases/download/v24.10.21.0/Sucrose_Bundle_.NET_Framework_4.8_x86_24.10.21.0.exe'
	}
}

Install-ChocolateyPackage @packageArgs