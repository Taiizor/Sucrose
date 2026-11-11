using System.IO;
using SSSHL = Sucrose.Shared.Space.Helper.Lockscreen;
using SSSHW = Sucrose.Shared.Space.Helper.WindowsImage;

namespace Sucrose.Commandog.Helper
{
    internal static class Lockscreen
    {
        public static async Task Apply(string Source, string ResultPath = null)
        {
            try
            {
                string Cache = SSSHL.GetCacheDirectory();

                (int Succeeded, int Failed, string Error) = await SSSHL.ApplyGifAsync(Cache, Source);

                if (Error is null)
                {
                    Console.WriteLine($"Lock screen applied to {Succeeded} destinations.");

                    WriteResult(ResultPath, $"OK|{Succeeded}|{Failed}");
                }
                else
                {
                    Console.WriteLine(Error);

                    WriteResult(ResultPath, $"ERR|{Error}");
                }
            }
            catch (Exception Exception)
            {
                Console.WriteLine(Exception.Message);

                WriteResult(ResultPath, $"ERR|{Exception.GetType().Name}: {Exception.Message}");
            }

            await Task.CompletedTask;
        }

        public static async Task Remove(string ResultPath = null)
        {
            try
            {
                string Cache = SSSHL.GetCacheDirectory();

                (int Succeeded, int Failed) = await SSSHL.RemoveAppliedAsync(Cache);

                Console.WriteLine($"Lock screen cache cleared: {Succeeded} removed, {Failed} failed.");

                WriteResult(ResultPath, $"OK|{Succeeded}|{Failed}");
            }
            catch (Exception Exception)
            {
                Console.WriteLine(Exception.Message);

                WriteResult(ResultPath, $"ERR|{Exception.GetType().Name}: {Exception.Message}");
            }

            await Task.CompletedTask;
        }

        public static async Task Feature(string Action, string ResultPath = null)
        {
            try
            {
                string Result = string.Equals(Action, "Enable", StringComparison.OrdinalIgnoreCase)
                    ? SSSHW.EnsureEnabled()
                    : SSSHW.EnsureDisabled();

                Console.WriteLine(Result);

                WriteResult(ResultPath, Result);
            }
            catch (Exception Exception)
            {
                Console.WriteLine(Exception.Message);

                WriteResult(ResultPath, $"ERR|{Exception.GetType().Name}: {Exception.Message}");
            }

            await Task.CompletedTask;
        }

        private static void WriteResult(string ResultPath, string Content)
        {
            if (string.IsNullOrEmpty(ResultPath))
            {
                return;
            }

            try
            {
                File.WriteAllText(ResultPath, Content.ReplaceLineEndings(" "));
            }
            catch
            {
            }
        }
    }
}
