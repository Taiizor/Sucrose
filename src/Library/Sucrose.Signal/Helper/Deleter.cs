using SMMRG = Sucrose.Memory.Manage.Readonly.General;

namespace Sucrose.Signal.Helper
{
    internal static class Deleter
    {
        public static async Task Delete(string filePath)
        {
            try
            {
                if (File.Exists(filePath))
                {
                    File.Delete(filePath);
                }
            }
            catch
            {
                try
                {
                    await Task.Delay(SMMRG.Randomise.Next(5, 50));

                    if (File.Exists(filePath))
                    {
                        File.Delete(filePath);
                    }
                }
                catch
                {
                    try
                    {
                        await Task.Delay(SMMRG.Randomise.Next(5, 50));

                        if (File.Exists(filePath))
                        {
                            File.Delete(filePath);
                        }
                    }
                    catch
                    {
                        //
                    }
                }
            }
        }
    }
}