using System.IO;
using SEAT = Skylark.Enum.AssemblyType;
using SHA = Skylark.Helper.Assemblies;
using SMR = Sucrose.Memory.Readonly;

namespace Sucrose.Space.Manage
{
    internal static class Internal
    {
        public static string Application => Path.Combine(Folder, SMR.ConsoleApplication);

        public static string Folder => Path.GetDirectoryName(SHA.Assemble(SEAT.Executing).Location);
    }
}