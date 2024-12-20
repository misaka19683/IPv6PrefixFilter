在编译时,pnet需要npcap这个库

对于当前会话，可以直接在命令行中设置环境变量
set LIB=C:\WpdPack\Lib\x64
或者是
$env:LIB = "C:\WpdPack\Lib\x64"




以下是编译WinDivert 2 Rust Wrapper的指导总结：

### 环境准备

1. **Rust工具链**：
   - 确保你的Rust版本使用的是MSVC工具链。

2. **WinPcap或Npcap**：
   - 安装WinPcap或Npcap。建议使用WinPcap 4.1.3版本。
   - 如果使用Npcap，请确保在安装时选择“Install Npcap in WinPcap API-compatible Mode”。

3. **Packet.lib库文件**：
   - 将WinPcap或者是Npcap开发者包中的Packet.lib文件放置在一个名为lib的目录中，该目录位于你的项目仓库的根目录下。 
   - 或者，你也可以将Packet.lib放在`%LIB%`或`$Env:LIB`环境变量列出的目录之一。
   - 对于64位工具链，Packet.lib通常位于`WpdPack/Lib/x64/Packet.lib`；对于32位工具链，位于`WpdPack/Lib/Packet.lib`。

### 编译步骤

1. **设置环境变量**：
   - 去下载windirert-2.2.0-win64.zip，解压后，将`dll`、`lib`和`sys`文件放置在一个文件夹中，并设置`WINDIVERT_PATH`环境变量指向该文件夹。
   - 使用`WINDIVERT_PATH`环境变量指定包含下载的dll、lib和sys文件的文件夹路径。
   - 如果启用了vendored特性，可以通过设置`WINDIVERT_DLL_OUTPUT`环境变量来保存生成的构建文件，以避免多次编译。

2. **编译WinDivert库文件**：
   - 如果启用了vendored特性，可以从源代码编译WinDivert的dll和lib文件。
   - 也可以通过启用static特性来编译，以便静态链接到WinDivert库。如果设置了`WINDIVERT_STATIC`环境变量，它将优先于crate特性。

3. **编译sys文件**：
   - 注意，sys文件必须始终提供，vendoring方法只会编译库文件。

4. **构建Rust Wrapper**：
   - 使用Cargo构建`windivert-sys`和`windivert`两个crates。

### 使用说明

- `windivert-sys`提供了与WinDivert用户模式库的原始绑定，API与原生库相同，具体细节请参考官方文档。
- `windivert`是建立在`windivert-sys`之上的，提供了更友好的Rust API和一些抽象。

### 注意事项

- WinDivert的dll期望对应的driver sys文件位于同一文件夹中。当你从官方网站下载dll、lib和sys文件时，它们会位于同一文件夹中，`windivert-sys`会在`WINDIVERT_PATH`提供的路径中搜索sys文件。

请确保按照上述步骤准备环境和编译项目，以确保WinDivert 2 Rust Wrapper能够正确编译和使用。
