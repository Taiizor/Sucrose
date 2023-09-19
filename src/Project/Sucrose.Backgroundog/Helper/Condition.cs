namespace Sucrose.Backgroundog.Helper
{
    internal static class Condition
    {
        public static async Task Start()
        {
            Console.WriteLine("Condition");
            //Performans şartları sağlanıp live motoru kapatıldıysa veya durdurulduysa
            //Live motoruna gRPC (Live.json) ile durdur mesajı gönderilir

            await Task.CompletedTask;
        }
    }
}
