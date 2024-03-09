namespace Rhythm.Contracts.Services;

public interface IPageService
{
    Type GetPageType(string key);

    Dictionary<string, string> GetPageKeys();
}
