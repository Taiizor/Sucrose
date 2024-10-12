using System.IO;
using SMR = Sucrose.Memory.Readonly;
using SSDECT = Sucrose.Shared.Dependency.Enum.CommandType;
using SSECCE = Skylark.Standard.Extension.Cryptology.CryptologyExtension;
using SSSHF = Sucrose.Shared.Space.Helper.Filing;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SSSHWE = Sucrose.Shared.Space.Helper.WatchException;
using SMMRG = Sucrose.Memory.Manage.Readonly.General;
using SSSMI = Sucrose.Shared.Space.Manage.Internal;

namespace Sucrose.Shared.Space.Helper
{
    internal static class Watchdog
    {
        public static string Read(string FilePath)
        {
            if (!Directory.Exists(Path.GetDirectoryName(FilePath)))
            {
                Directory.CreateDirectory(Path.GetDirectoryName(FilePath));
            }

            return SSSHF.ReadStream(FilePath);
        }

        public static void Write(string FilePath, string FileContent)
        {
            if (!Directory.Exists(Path.GetDirectoryName(FilePath)))
            {
                Directory.CreateDirectory(Path.GetDirectoryName(FilePath));
            }

            SSSHF.WriteStream(FilePath, FileContent);
        }

        public static void Start(string Application, Exception Exception, string Path)
        {
            if (Check())
            {
                SSSHP.Run(SSSMI.Commandog, $"{SMMRG.StartCommand}{SSDECT.Watchdog}{SMMRG.ValueSeparator}{SSSMI.Watchdog}{SMMRG.ValueSeparator}{Encrypt(Application, Exception, Path)}");
            }
        }

        public static void Start(string Application, Exception Exception, string Path, string Source, string Text)
        {
            if (Check())
            {
                if (string.IsNullOrEmpty(Source) || string.IsNullOrEmpty(Text))
                {
                    Start(Application, Exception, Path);
                }
                else
                {
                    SSSHP.Run(SSSMI.Commandog, $"{SMMRG.StartCommand}{SSDECT.Watchdog}{SMMRG.ValueSeparator}{SSSMI.Watchdog}{SMMRG.ValueSeparator}{Encrypt(Application, Exception, Path, Source, Text)}");
                }
            }
        }

        private static bool Check()
        {
            return File.Exists(SSSMI.Commandog) && File.Exists(SSSMI.Watchdog);
        }

        private static string Encrypt(string Application, Exception Exception, string Path)
        {
            return SSECCE.TextToBase($"{Application}{SMMRG.ValueSeparator}{SSSHWE.Convert(Exception)}{SMMRG.ValueSeparator}{Path}");
        }

        private static string Encrypt(string Application, Exception Exception, string Path, string Source, string Text)
        {
            return SSECCE.TextToBase($"{Application}{SMMRG.ValueSeparator}{SSSHWE.Convert(Exception)}{SMMRG.ValueSeparator}{Path}{SMMRG.ValueSeparator}{Source}{SMMRG.ValueSeparator}{Text}");
        }
    }
}