using SDECT = Sucrose.Dependency.Enum.CommandsType;
using SMR = Sucrose.Memory.Readonly;
using SSHP = Sucrose.Space.Helper.Processor;
using SSMI = Sucrose.Space.Manage.Internal;

namespace Sucrose.Tray.Command
{
    internal static class Interface
    {
        public static void Command()
        {
#if TRAY_ICON_WPF
            SSHP.Run(SSMI.Commandog, $"{SMR.StartCommand}{SDECT.Interface}{SMR.ValueSeparator}{SSMI.WPFUserInterface}");
#elif TRAY_ICON_WinForms
            SSHP.Run(SSMI.Commandog, $"{SMR.StartCommand}{SDECT.Interface}{SMR.ValueSeparator}{SSMI.WinFormsUserInterface}");
#endif
        }
    }
}