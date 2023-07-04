using System.IO;
using SEAT = Skylark.Enum.AssemblyType;
using SHA = Skylark.Helper.Assemblies;
using SMR = Sucrose.Memory.Readonly;

namespace Sucrose.Space.Manage
{
    internal static class Internal
    {
        public static string Folder => Path.Combine(Path.GetDirectoryName(SHA.Assemble(SEAT.Executing).Location), @"..\");

        public static string CommandLine => Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMR.CommandLine), SMR.CommandLine);

        public static string WPFUserInterface => Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMR.WPFUserInterface), SMR.WPFUserInterface);

        public static string WinFormsUserInterface => Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMR.WinFormsUserInterface), SMR.WinFormsUserInterface);
    }
}