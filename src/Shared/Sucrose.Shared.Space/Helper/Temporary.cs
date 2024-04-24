using System.IO;
using SECNT = Skylark.Enum.ClearNumericType;
using SEMST = Skylark.Enum.ModeStorageType;
using SEST = Skylark.Enum.StorageType;
using SHN = Skylark.Helper.Numeric;
using SMR = Sucrose.Memory.Readonly;
using SSDEACT = Sucrose.Shared.Dependency.Enum.ArgumentCommandsType;
using SSESSE = Skylark.Standard.Extension.Storage.StorageExtension;
using SSLHK = Sucrose.Shared.Live.Helper.Kill;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SSSMI = Sucrose.Shared.Space.Manage.Internal;
using SSSSS = Skylark.Struct.Storage.StorageStruct;

namespace Sucrose.Shared.Space.Helper
{
    internal static class Temporary
    {
        public static async Task Delete(string Destination, string Application)
        {
            SSLHK.Stop();

            SSSHP.Kill(SMR.Undo);
            SSSHP.Kill(SMR.Portal);
            SSSHP.Kill(SMR.Update);
            SSSHP.Kill(SMR.Wizard);
            SSSHP.Kill(SMR.Launcher);
            SSSHP.Kill(SMR.Property);
            SSSHP.Kill(SMR.Watchdog);
            SSSHP.Kill(SMR.Backgroundog);

            await Task.Delay(TimeSpan.FromSeconds(3));

            if (Directory.Exists(Destination))
            {
                Directory.Delete(Destination, true);
            }
            else
            {
                Directory.CreateDirectory(Destination);
            }

            await Task.Delay(TimeSpan.FromSeconds(1));

            SSSHP.Run(Application);
            SSSHP.Run(SSSMI.Portal, $"{SSDEACT.SystemSetting}");

            await Task.CompletedTask;
        }

        public static async Task<string> Size(string Destination)
        {
            long Total = 0;

            if (Directory.Exists(Destination))
            {
                string[] Files = Directory.GetFiles(Destination, "*", SearchOption.AllDirectories);

                foreach (string Record in Files)
                {
                    FileInfo Info = new(Record);
                    Total += Info.Length;
                }

                SSSSS Result = await SSESSE.AutoConvertAsync(Total, SEST.Byte, SEMST.Palila);

                return await SHN.NumeralAsync(Result.Value, true, true, 2, '0', SECNT.None) + " " + Result.Short;
            }
            else
            {
                return await Task.FromResult("0 b");
            }
        }

        public static bool Check(string Destination)
        {
            if (Directory.Exists(Destination))
            {
                return true;
            }
            else if (File.Exists(Destination))
            {
                return true;
            }
            else
            {
                return false;
            }
        }
    }
}