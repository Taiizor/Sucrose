$ErrorActionPreference = 'Stop'

Get-Process 'Sucrose*' | % {
	Write-Output ('Closing: {0}' -f $_.ProcessName)
	Stop-Process -InputObject $_ -Force
}

$packageArgs = @{
	packageName    = 'Sucrose Wallpaper Engine'
	checksumType   = 'sha256'
	fileType       = 'exe'
	silentArgs     = '/s'
	validExitCodes = @(0)
}

if ($env:PROCESSOR_ARCHITECTURE -eq 'ARM64') {
	$packageArgs['checksum'] = 'ECB99C3152C0B2709679ED4936E96EB23AC0421B0385B471CC97CA3C8241E23F'
	$packageArgs['url'] = 'https://github.com/Taiizor/Sucrose/releases/download/v26.11.22.0/Sucrose_Bundle_.NET_Framework_4.8_ARM64_26.11.22.0.exe'
} else {
	if ([Environment]::Is64BitOperatingSystem) {
		if ([System.Environment]::Is64BitProcess) {
			$packageArgs['checksum'] = '99F9DB10F85739A63CD0BF31DD8CCCDF55833376B219BC580950387CC9BD78FD'
			$packageArgs['url'] = 'https://github.com/Taiizor/Sucrose/releases/download/v26.11.22.0/Sucrose_Bundle_.NET_Framework_4.8_x64_26.11.22.0.exe'
		} else {
			$packageArgs['checksum'] = '10AFA2110AD35F3C3C13876D5DF476182F56B67747E7CE24E17618FBE6BBAEBC'
			$packageArgs['url'] = 'https://github.com/Taiizor/Sucrose/releases/download/v26.11.22.0/Sucrose_Bundle_.NET_Framework_4.8_x86_26.11.22.0.exe'
		}
	} else {
		$packageArgs['checksum'] = '10AFA2110AD35F3C3C13876D5DF476182F56B67747E7CE24E17618FBE6BBAEBC'
		$packageArgs['url'] = 'https://github.com/Taiizor/Sucrose/releases/download/v26.11.22.0/Sucrose_Bundle_.NET_Framework_4.8_x86_26.11.22.0.exe'
	}
}

Install-ChocolateyPackage @packageArgs