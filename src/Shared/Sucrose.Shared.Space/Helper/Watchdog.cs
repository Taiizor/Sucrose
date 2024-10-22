using System.Collections;
using System.IO;
using SMMRG = Sucrose.Memory.Manage.Readonly.General;
using SSDECT = Sucrose.Shared.Dependency.Enum.CommandType;
using SSECCE = Skylark.Standard.Extension.Cryptology.CryptologyExtension;
using SSSEWE = Sucrose.Shared.Space.Extension.WatchException;
using SSSHF = Sucrose.Shared.Space.Helper.Filing;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SSSMI = Sucrose.Shared.Space.Manage.Internal;
using SSWHD = Sucrose.Shared.Watchdog.Helper.Dataset;

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
                Exception = Data(Exception);

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
                    Exception = Data(Exception);

                    SSSHP.Run(SSSMI.Commandog, $"{SMMRG.StartCommand}{SSDECT.Watchdog}{SMMRG.ValueSeparator}{SSSMI.Watchdog}{SMMRG.ValueSeparator}{Encrypt(Application, Exception, Path, Source, Text)}");
                }
            }
        }

        private static bool Check()
        {
            return File.Exists(SSSMI.Commandog) && File.Exists(SSSMI.Watchdog);
        }

        private static Exception Data(Exception Exception)
        {
            if (Exception != null && SSWHD.Any())
            {
                foreach (DictionaryEntry Entry in SSWHD.Get())
                {
                    Exception.Data.Add(Entry.Key, Entry.Value);
                }

                SSWHD.Clear();
            }

            return Exception;
        }

        private static string Encrypt(string Application, Exception Exception, string Path)
        {
            return SSECCE.TextToBase($"{Application}{SMMRG.ValueSeparator}{SSSEWE.Convert(Exception)}{SMMRG.ValueSeparator}{Path}");
        }

        private static string Encrypt(string Application, Exception Exception, string Path, string Source, string Text)
        {
            return SSECCE.TextToBase($"{Application}{SMMRG.ValueSeparator}{SSSEWE.Convert(Exception)}{SMMRG.ValueSeparator}{Path}{SMMRG.ValueSeparator}{Source}{SMMRG.ValueSeparator}{Text}");
        }
    }
}