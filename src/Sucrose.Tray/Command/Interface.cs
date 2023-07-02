using System.IO;
using SMR = Sucrose.Memory.Readonly;
using SSECT = Sucrose.Space.Enum.CommandsType;
using SSHC = Sucrose.Space.Helper.Command;
using SHA = Skylark.Helper.Assemblies;
using SEAT = Skylark.Enum.AssemblyType;

namespace Sucrose.Tray.Command
{
    public static class Interface
    {
        public static void Command()
        {
            string Folder = Path.GetDirectoryName(SHA.Assemble(SEAT.Executing).Location);

#if TRAY_ICON_WPF
            SSHC.Run(Path.Combine(Folder, SMR.ConsoleApplication), $"{SMR.StartCommand}{SSECT.Interface}{SMR.ValueSeparator}{Path.Combine(Folder, SMR.WPFApplication)}");
#elif TRAY_ICON_WinForms
            SSHC.Run(Path.Combine(Folder, SMR.ConsoleApplication), $"{SMR.StartCommand}{SSECT.Interface}{SMR.ValueSeparator}{Path.Combine(Folder, SMR.WinFormsApplication)}");
#endif
        }
    }
}