using System.IO;
using SEAT = Skylark.Enum.AssemblyType;
using SHV = Skylark.Helper.Versionly;
using SMR = Sucrose.Memory.Readonly;
using SSDECT = Sucrose.Shared.Dependency.Enum.CommandsType;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SSSMI = Sucrose.Shared.Space.Manage.Internal;

namespace Sucrose.Shared.Launcher.Command
{
    internal static class Version
    {
        public static void Command()
        {
            SSSHP.Run(SSSMI.Commandog, $"{SMR.StartCommand}{SSDECT.Versioning}{SMR.ValueSeparator}{Path.Combine(SMR.LocalAppDataPath, SMR.AppName)}{SMR.ValueSeparator}{SHV.Auto(SEAT.Entry)}{SMR.ValueSeparator}{false}");
        }
    }
}