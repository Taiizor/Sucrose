namespace Sucrose.Portal.Contracts.Services;

public interface IActivationService
{
    Task ActivateAsync(object activationArgs);
}