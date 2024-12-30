## WinDivert Installation Guide

1. Download the [WinDivert: Windows Packet Divert](https://reqrypt.org/windivert.html) file: [WinDivert-2.2.2-A.zip](https://reqrypt.org/download/WinDivert-2.2.2-A.zip).

2. Locate the `x64\WinDivert64.sys` file.

3. Copy this file to `C:\Windows\System32\drivers`.

4. Run the following commands in a `cmd` terminal with administrator privileges:

   ```cmd
   C:\Windows\System32>sc create WinDivert binPath= "C:\Windows\System32\drivers\WinDivert64.sys" type= kernel start= system

   [SC] CreateService SUCCESS
   ```

   ```cmd
   C:\Windows\System32>sc query WinDivert

   SERVICE_NAME: WinDivert
           TYPE               : 1  KERNEL_DRIVER
           STATE              : 4  RUNNING
                                   (STOPPABLE, NOT_PAUSABLE, IGNORES_SHUTDOWN)
           WIN32_EXIT_CODE    : 0  (0x0)
           SERVICE_EXIT_CODE  : 0  (0x0)
           CHECKPOINT         : 0x0
           WAIT_HINT          : 0x0
   ```

5. Success!

