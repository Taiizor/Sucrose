using System.IO;
using SMR = Sucrose.Memory.Readonly;
using SSECT = Sucrose.Space.Enum.CommandsType;
using SSHC = Sucrose.Space.Helper.Command;
using SSMI = Sucrose.Space.Manage.Internal;

namespace Sucrose.Tray.Command
{
    internal static class Interface
    {
        public static void Command()
        {
#if TRAY_ICON_WPF
            SSHC.Run(SSMI.Application, $"{SMR.StartCommand}{SSECT.Interface}{SMR.ValueSeparator}{Path.Combine(SSMI.Folder, SMR.WPFApplication)}");
#elif TRAY_ICON_WinForms
            SSHC.Run(SSMI.Application, $"{SMR.StartCommand}{SSECT.Interface}{SMR.ValueSeparator}{Path.Combine(SSMI.Folder, SMR.WinFormsApplication)}");
#endif
        }
    }
}