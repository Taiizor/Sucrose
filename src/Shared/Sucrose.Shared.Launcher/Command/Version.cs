using System.IO;
using SEAT = Skylark.Enum.AssemblyType;
using SHV = Skylark.Helper.Versionly;
using SMMRP = Sucrose.Memory.Manage.Readonly.Path;
using SMR = Sucrose.Memory.Readonly;
using SSDECT = Sucrose.Shared.Dependency.Enum.CommandType;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SSSMI = Sucrose.Shared.Space.Manage.Internal;

namespace Sucrose.Shared.Launcher.Command
{
    internal static class Version
    {
        public static void Command()
        {
            SSSHP.Run(SSSMI.Commandog, $"{SMR.StartCommand}{SSDECT.Versioning}{SMR.ValueSeparator}{Path.Combine(SMMRP.LocalApplicationData, SMR.AppName)}{SMR.ValueSeparator}{SHV.Auto(SEAT.Entry)}{SMR.ValueSeparator}{false}");
        }
    }
}