using SMR = Sucrose.Memory.Readonly;
using SSDECT = Sucrose.Shared.Dependency.Enum.CommandsType;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SSSMI = Sucrose.Shared.Space.Manage.Internal;

namespace Sucrose.Shared.Launcher.Command
{
    internal static class Update
    {
        public static void Command(bool State = true)
        {
            if (State)
            {
                SSSHP.Run(SSSMI.Commandog, $"{SMR.StartCommand}{SSDECT.Update}{SMR.ValueSeparator}{SSSMI.Update}");
            }
            else
            {
                SSSHP.Run(SSSMI.Commandog, $"{SMR.StartCommand}{SSDECT.Update}{SMR.ValueSeparator}{SSSMI.Update}{SMR.ValueSeparator}{State}");
            }
        }
    }
}