using SBHI = Sucrose.Backgroundog.Helper.Initialize;
using SBMI = Sucrose.Backgroundog.Manage.Internal;
using SSSHL = Sucrose.Shared.Space.Helper.Live;

namespace Sucrose.Backgroundog.Helper
{
    internal static class Attempt
    {
        public static async Task Start()
        {
            Console.WriteLine("Deneme");
            int MaxAttempts = 5;
            bool Success = false;
            int IntervalInSeconds = 1;

            for (int Attempt = 0; Attempt <= MaxAttempts; Attempt++)
            {
                Success = SSSHL.Run();

                if (Success)
                {
                    Console.WriteLine("İşlem başarılı!");
                    break;
                }

                await Task.Delay(IntervalInSeconds * 1000);
            }

            if (!Success)
            {
                Console.WriteLine("İşlem başarısız oldu ve 5 deneme sonunda hala true dönmedi.");
                SBMI.Exit = false;
                SBHI.Stop();
            }
        }
    }
}
