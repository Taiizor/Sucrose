namespace Sucrose.Backgroundog.Helper
{
    internal static class Performance
    {
        public static async Task Start()
        {
            Console.WriteLine("Run");
            //Performans şartları kontrol edilecek...

            await Task.CompletedTask;
        }
    }
}