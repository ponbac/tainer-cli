#!/usr/bin/env pwsh
param(
  # TODO: change this to 'latest' when Bun for Windows is stable.
  [String]$Version = "latest",
  # Forces installing the baseline build regardless of what CPU you are actually using.
  [Switch]$ForceBaseline = $false,
  # Skips adding the bun.exe directory to the user's %PATH%
  [Switch]$NoPathUpdate = $false,
  # Skips adding the bun to the list of installed programs
  [Switch]$NoRegisterInstallation = $false,
  # Skips installing powershell completions to your profile
  [Switch]$NoCompletions = $false,

  # Debugging: Always download with 'Invoke-RestMethod' instead of 'curl.exe'
  [Switch]$DownloadWithoutCurl = $false
);

# This corresponds to .win10_rs5 in build.zig
$MinBuild = 17763;
$MinBuildName = "Windows 10 1809"

$WinVer = [System.Environment]::OSVersion.Version
if ($WinVer.Major -lt 10 -or ($WinVer.Major -eq 10 -and $WinVer.Build -lt $MinBuild)) {
  Write-Warning "Requires at least ${MinBuildName} or newer.`n`nThe install will still continue but it may not work.`n"
  return 1
}

$ErrorActionPreference = "Stop"

# These three environment functions are roughly copied from https://github.com/prefix-dev/pixi/pull/692
# They are used instead of `SetEnvironmentVariable` because of unwanted variable expansions.
function Publish-Env {
  if (-not ("Win32.NativeMethods" -as [Type])) {
    Add-Type -Namespace Win32 -Name NativeMethods -MemberDefinition @"
[DllImport("user32.dll", SetLastError = true, CharSet = CharSet.Auto)]
public static extern IntPtr SendMessageTimeout(
    IntPtr hWnd, uint Msg, UIntPtr wParam, string lParam,
    uint fuFlags, uint uTimeout, out UIntPtr lpdwResult);
"@
  }
  $HWND_BROADCAST = [IntPtr] 0xffff
  $WM_SETTINGCHANGE = 0x1a
  $result = [UIntPtr]::Zero
  [Win32.NativeMethods]::SendMessageTimeout($HWND_BROADCAST,
    $WM_SETTINGCHANGE,
    [UIntPtr]::Zero,
    "Environment",
    2,
    5000,
    [ref] $result
  ) | Out-Null
}

function Write-Env {
  param([String]$Key, [String]$Value)

  $RegisterKey = Get-Item -Path 'HKCU:'

  $EnvRegisterKey = $RegisterKey.OpenSubKey('Environment', $true)
  if ($null -eq $Value) {
    $EnvRegisterKey.DeleteValue($Key)
  } else {
    $RegistryValueKind = if ($Value.Contains('%')) {
      [Microsoft.Win32.RegistryValueKind]::ExpandString
    } elseif ($EnvRegisterKey.GetValue($Key)) {
      $EnvRegisterKey.GetValueKind($Key)
    } else {
      [Microsoft.Win32.RegistryValueKind]::String
    }
    $EnvRegisterKey.SetValue($Key, $Value, $RegistryValueKind)
  }

  Publish-Env
}

function Get-Env {
  param([String] $Key)

  $RegisterKey = Get-Item -Path 'HKCU:'
  $EnvRegisterKey = $RegisterKey.OpenSubKey('Environment')
  $EnvRegisterKey.GetValue($Key, $null, [Microsoft.Win32.RegistryValueOptions]::DoNotExpandEnvironmentNames)
}

# The installation of bun is it's own function so that in the unlikely case the $IsBaseline check fails, we can do a recursive call.
# There are also lots of sanity checks out of fear of anti-virus software or other weird Windows things happening.
function Install-Tainer {
  param(
    [bool]$ForceBaseline = $False
  );

  $IsBaseline = $ForceBaseline
  if (!$IsBaseline) {
    $IsBaseline = !( `
      Add-Type -MemberDefinition '[DllImport("kernel32.dll")] public static extern bool IsProcessorFeaturePresent(int ProcessorFeature);' `
        -Name 'Kernel32' -Namespace 'Win32' -PassThru `
    )::IsProcessorFeaturePresent(40);
  }

  $TainerRoot = if ($env:TAINER_INSTALL) { $env:TAINER_INSTALL } else { "${Home}\.tainer" }
  $TainerBin = mkdir -Force "${TainerRoot}\bin"

  try {
    Remove-Item "${TainerBin}\tainer.exe" -Force
  } catch [System.Management.Automation.ItemNotFoundException] {
    # ignore
  } catch [System.UnauthorizedAccessException] {
    $openProcesses = Get-Process -Name bun | Where-Object { $_.Path -eq "${TainerBin}\tainer.exe" }
    if ($openProcesses.Count -gt 0) {
      Write-Output "Install Failed - An older installation exists and is open. Please close open tainer-cli processes and try again."
      return 1
    }
    Write-Output "Install Failed - An unknown error occurred while trying to remove the existing installation"
    Write-Output $_
    return 1
  } catch {
    Write-Output "Install Failed - An unknown error occurred while trying to remove the existing installation"
    Write-Output $_
    return 1
  }

  $Target = "tainer-cli-win64"
  $BaseURL = "https://github.com/ponbac/tainer-cli/releases"
  $URL = "$BaseURL/$(if ($Version -eq "latest") { "latest/download" } else { "download/$Version" })/$Target.zip"

  $ZipPath = "${TainerBin}\$Target.zip"

  $null = mkdir -Force $TainerBin
  Remove-Item -Force $ZipPath -ErrorAction SilentlyContinue

  # curl.exe is faster than PowerShell 5's 'Invoke-WebRequest'
  # note: 'curl' is an alias to 'Invoke-WebRequest'. so the exe suffix is required
  if (-not $DownloadWithoutCurl) {
    curl.exe "-#SfLo" "$ZipPath" "$URL" 
  }
  if ($DownloadWithoutCurl -or ($LASTEXITCODE -ne 0)) {
    Write-Warning "The command 'curl.exe $URL -o $ZipPath' exited with code ${LASTEXITCODE}`nTrying an alternative download method..."
    try {
      # Use Invoke-RestMethod instead of Invoke-WebRequest because Invoke-WebRequest breaks on
      # some machines, see 
      Invoke-RestMethod -Uri $URL -OutFile $ZipPath
    } catch {
      Write-Output "Install Failed - could not download $URL"
      Write-Output "The command 'Invoke-RestMethod $URL -OutFile $ZipPath' exited with code ${LASTEXITCODE}`n"
      return 1
    }
  }

  if (!(Test-Path $ZipPath)) {
    Write-Output "Install Failed - could not download $URL"
    Write-Output "The file '$ZipPath' does not exist. Did an antivirus delete it?`n"
    return 1
  }

  try {
    $lastProgressPreference = $global:ProgressPreference
    $global:ProgressPreference = 'SilentlyContinue';
    Expand-Archive "$ZipPath" "$TainerBin" -Force
    $global:ProgressPreference = $lastProgressPreference
    # MARKER!
    if (!(Test-Path "${TainerBin}\tainer.exe")) {
      throw "The file '${TainerBin}\tainer.exe' does not exist. Download is corrupt or intercepted Antivirus?`n"
    }
  } catch {
    Write-Output "Install Failed - could not unzip $ZipPath"
    Write-Error $_
    return 1
  }

  Remove-Item $ZipPath -Force

  if ($LASTEXITCODE -eq 1073741795) { # STATUS_ILLEGAL_INSTRUCTION
    Write-Output "Install Failed - tainer.exe is not compatible with your CPU. This should have been detected before downloading.`n"
    return 1
  }
  # '-1073741515' was spotted in the wild, but not clearly documented as a status code:
  # https://discord.com/channels/876711213126520882/1149339379446325248/1205194965383250081
  # http://community.sqlbackupandftp.com/t/error-1073741515-solved/1305
  if (($LASTEXITCODE -eq 3221225781) -or ($LASTEXITCODE -eq -1073741515)) # STATUS_DLL_NOT_FOUND
  { 
    Write-Output "Install Failed - You are missing a DLL required to run tainer.exe"
    Write-Output "This can be solved by installing the Visual C++ Redistributable from Microsoft:`nSee https://learn.microsoft.com/cpp/windows/latest-supported-vc-redist`nDirect Download -> https://aka.ms/vs/17/release/vc_redist.x64.exe`n`n"
    Write-Output "The command '${TainerBin}\tainer.exe --revision' exited with code ${LASTEXITCODE}`n"
    return 1
  }
  if ($LASTEXITCODE -ne 0) {
    Write-Output "Install Failed - could not verify tainer.exe"
    Write-Output "The command '${TainerBin}\tainer.exe --revision' exited with code ${LASTEXITCODE}`n"
    return 1
  }

  $env:IS_BUN_AUTO_UPDATE = $null
  $env:BUN_NO_INSTALL_COMPLETIONS = $null

  $C_RESET = [char]27 + "[0m"
  $C_GREEN = [char]27 + "[1;32m"

  Write-Output "${C_GREEN}Envirotainer-CLI was installed successfully!${C_RESET}"
  Write-Output "The binary is located at ${TainerBin}\tainer.exe`n"

  $hasExistingOther = $false;
  try {
    $existing = Get-Command tainer -ErrorAction
    if ($existing.Source -ne "${TainerBin}\tainer.exe") {
      Write-Warning "Note: Another tainer.exe is already in %PATH% at $($existing.Source)`nTyping 'tainer' in your terminal will not use what was just installed.`n"
      $hasExistingOther = $true;
    }
  } catch {}

  if (-not $NoRegisterInstallation) {
    $rootKey = $null
    try {
      $RegistryKey = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\Tainer"  
      $rootKey = New-Item -Path $RegistryKey -Force
      New-ItemProperty -Path $RegistryKey -Name "DisplayName" -Value "Tainer" -PropertyType String -Force | Out-Null
      New-ItemProperty -Path $RegistryKey -Name "InstallLocation" -Value "${TainerRoot}" -PropertyType String -Force | Out-Null
      New-ItemProperty -Path $RegistryKey -Name "DisplayIcon" -Value $TainerBin\tainer.exe -PropertyType String -Force | Out-Null
      New-ItemProperty -Path $RegistryKey -Name "UninstallString" -Value "powershell -c `"& `'$TainerRoot\uninstall.ps1`' -PauseOnError`" -ExecutionPolicy Bypass" -PropertyType String -Force | Out-Null
    } catch {
      if ($null -ne $rootKey) {
        Remove-Item -Path $RegistryKey -Force
      }
    }
  }

  if(!$hasExistingOther) {
    # Only try adding to path if there isn't already a tainer.exe in the path
    $Path = (Get-Env -Key "Path") -split ';'
    if ($Path -notcontains $TainerBin) {
      if (-not $NoPathUpdate) {
        $Path += $TainerBin
        Write-Env -Key 'Path' -Value ($Path -join ';')
        $env:PATH = $Path;
      } else {
        Write-Output "Skipping adding '${TainerBin}' to the user's %PATH%`n"
      }
    }

    Write-Output "To get started, restart your terminal/editor, then type `"tainer`"`n"
  }

  $LASTEXITCODE = 0;
}

Install-Tainer -Version $Version -ForceBaseline $ForceBaseline