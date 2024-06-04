$serviceName = "Sucrose Servicer"
$serviceExePath = "Sucrose.Servicer.exe"

# Hizmet zaten varsa durdur ve kaldır
if (Get-Service -Name $serviceName -ErrorAction SilentlyContinue) {
    Stop-Service -Name $serviceName
    sc.exe delete $serviceName
}

# Hizmeti yükle
sc.exe create $serviceName binPath= $serviceExePath

# Hizmeti başlat
Start-Service -Name $serviceName