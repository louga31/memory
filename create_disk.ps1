function Copy-File ([System.String] $sourceFile, [System.String] $destinationFile, [Switch] $overWrite) {

    if ($sourceFile -notlike "filesystem::*") {
        $sourceFile = "filesystem::$sourceFile" 
    }

    if ($destinationFile -notlike "filesystem::*") {
        $destinationFile = "filesystem::$destinationFile" 
    }

    $destinationFolder = $destinationFile.Replace($destinationFile.Split("/")[-1],"")

    if (!(Test-Path -path $destinationFolder)) {
        New-Item $destinationFolder -Type Directory
    }

    try {
        Copy-Item -Path $sourceFile -Destination $destinationFile -Recurse -Force
        Return $true 
    } catch [System.IO.IOException] {
        # If overwrite enabled, then delete the item from the destination, and try again:
        if ($overWrite) {
            try {
                Remove-Item -Path $destinationFile -Recurse -Force        
                Copy-Item -Path $sourceFile -Destination $destinationFile -Recurse -Force 
                Return $true
            } catch {
                Write-Error -Message "[Copy-File] Overwrite error occurred!`n$_" -ErrorAction SilentlyContinue
                #$PSCmdlet.WriteError($Global:Error[0])
                Return $false
            }
        } else {
            Write-Error -Message "[Copy-File] File already exists!" -ErrorAction SilentlyContinue
            #$PSCmdlet.WriteError($Global:Error[0])
            Return $false
        }
    } catch {
        Write-Error -Message "[Copy-File] File move failed!`n$_" -ErrorAction SilentlyContinue
        #$PSCmdlet.WriteError($Global:Error[0]) 
        Return $false
    } 
}

if (!$args[0]) {
    $configuration = "debug";
} else {
    $configuration = "release";
}

Copy-File "./target/x86_64-unknown-uefi/$configuration/memory.efi" "_efi/Driver.efi" -overWrite;
Copy-File "./bootx64.efi" "_efi/EFI/Boot/Bootx64.efi" -overWrite;
# New-VHD -path "efi.vhd" -SizeBytes 24MB | Mount-VHD -Passthru | Initialize-Disk -PassThru -PartitionStyle GPT | New-Partition -AssignDriveLetter -UseMaximumSize | Format-Volume -FileSystem FAT -Confirm:$false -Force
$MountedDisk = Mount-VHD -Path "efi.vhd" -Passthru

[String]$DriveLetter = ($MountedDisk | Get-Disk | Get-Partition | Where-Object {$_.Type -eq "Basic"} | Select-Object -ExpandProperty DriveLetter) + ":"
$DriveLetter = $DriveLetter.Replace(' ','')

Format-Volume -DriveLetter $DriveLetter.Replace(':', '') -FileSystem FAT -Confirm:$false -Force

Copy-File "_efi/Driver.efi" $DriveLetter/Driver.efi -overWrite
Copy-File "_efi/EFI/Boot/Bootx64.efi" $DriveLetter/EFI/Boot/Bootx64.efi -overWrite

Dismount-VHD -Path "efi.vhd"

qemu-img convert -O vmdk efi.vhd efi.vmdk