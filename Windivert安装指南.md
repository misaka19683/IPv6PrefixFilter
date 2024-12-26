Windivert安装指南

1. 下载[WinDivert: Windows Packet Divert](https://reqrypt.org/windivert.html)的[WinDivert-2.2.2-A.zip](https://reqrypt.org/download/WinDivert-2.2.2-A.zip)文件

2. 找到x64\WinDivert64.sys文件

3. 将此文件复制到C:\Windows\System32\drivers下

4. 使用管理员权限的cmd运行

   ```cmd
   C:\Windows\System32>sc create WinDivert binPath= "C:\Windows\System32\drivers\WinDivert64.sys" type= kernel start= system
   
   [SC] CreateService 成功
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

   

5. 成功!