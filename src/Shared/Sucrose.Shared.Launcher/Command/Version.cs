﻿using System.IO;
using SEAT = Skylark.Enum.AssemblyType;
using SHV = Skylark.Helper.Versionly;
using SMMRG = Sucrose.Memory.Manage.Readonly.General;
using SMMRP = Sucrose.Memory.Manage.Readonly.Path;
using SSDECT = Sucrose.Shared.Dependency.Enum.CommandType;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SSSMI = Sucrose.Shared.Space.Manage.Internal;

namespace Sucrose.Shared.Launcher.Command
{
    internal static class Version
    {
        public static void Command()
        {
            SSSHP.Run(SSSMI.Commandog, $"{SMMRG.StartCommand}{SSDECT.Versioning}{SMMRG.ValueSeparator}{Path.Combine(SMMRP.LocalApplicationData, SMMRG.AppName)}{SMMRG.ValueSeparator}{SHV.Auto(SEAT.Entry)}{SMMRG.ValueSeparator}{false}");
        }
    }
}