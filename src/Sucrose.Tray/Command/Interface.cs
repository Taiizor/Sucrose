using System.IO;
using System.Reflection;
using SMR = Sucrose.Memory.Readonly;
using SSECT = Sucrose.Space.Enum.CommandsType;
using SSHC = Sucrose.Space.Helper.Command;

namespace Sucrose.Tray.Command
{
    public static class Interface
    {
        public static void Command()
        {
#if TRAY_ICON_WPF
            string Folder = Path.GetDirectoryName(Assembly.GetExecutingAssembly().Location);

            SSHC.Run(Path.Combine(Folder, SMR.ConsoleApplication), $"{SMR.StartCommand}{SSECT.Interface}{SMR.ValueSeparator}{Path.Combine(Folder, SMR.WPFApplication)}");
#elif TRAY_ICON_WinForms
            string Folder = Path.GetDirectoryName(Assembly.GetExecutingAssembly().Location);

            SSHC.Run(Path.Combine(Folder, SMR.ConsoleApplication), $"{SMR.StartCommand}{SSECT.Interface}{SMR.ValueSeparator}{Path.Combine(Folder, SMR.WinFormsApplication)}");
#endif
        }
    }
}